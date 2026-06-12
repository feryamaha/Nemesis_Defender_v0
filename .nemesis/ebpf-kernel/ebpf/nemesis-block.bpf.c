// SPDX-License-Identifier: GPL-2.0
//
// nemesis-block.bpf.c — BPF LSM enforcement via lsm/bprm_check_security
// Substitui o backend kprobe anterior.
// Requer: bpf ativo em /sys/kernel/security/lsm (adicionar ao cmdline do GRUB).
// Não usa bpf_override_return nem ALLOW_ERROR_INJECTION.

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>
#include <bpf/bpf_endian.h>

#ifndef EPERM
#define EPERM 1
#endif

#include "nemesis_maps.h"

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 256);
    __type(key, struct command_key);
    __type(value, __u8);
} blocked_commands SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 24);
} events SEC(".maps");

// Mapa para armazenar o cgroup_id do agente Nemesis
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __uint(max_entries, 1);
    __type(key, u32);
    __type(value, u64);
} agent_cgroup_map SEC(".maps");

// Allowlist de egress IPv4 (LPM trie: longest-prefix match em CIDR)
struct {
    __uint(type, BPF_MAP_TYPE_LPM_TRIE);
    __uint(max_entries, 1024);
    __type(key, struct egress_v4_key);
    __type(value, struct egress_val);
    __uint(map_flags, BPF_F_NO_PREALLOC);
} egress_allow_v4 SEC(".maps");

// Allowlist de egress IPv6
struct {
    __uint(type, BPF_MAP_TYPE_LPM_TRIE);
    __uint(max_entries, 1024);
    __type(key, struct egress_v6_key);
    __type(value, struct egress_val);
    __uint(map_flags, BPF_F_NO_PREALLOC);
} egress_allow_v6 SEC(".maps");

// Flag de enforce (índice 0): 0 = observar/permitir, 1 = impor deny-by-default
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __uint(max_entries, 1);
    __type(key, u32);
    __type(value, u8);
} egress_enforce SEC(".maps");

// Tamanho máximo de path lido do kernel — menor para caber no stack BPF
#define BPF_PATH_LEN 128

SEC("lsm/bprm_check_security")
int BPF_PROG(nemesis_check_exec, struct linux_binprm *bprm, int ret)
{
    struct command_key key = {};
    __u8 *blocked;
    struct nemesis_event *event;
    __u64 pid_tgid;
    const char *filename;
    char path[BPF_PATH_LEN] = {};
    int i, slash = -1;

    // Respeita decisão de LSM anterior na cadeia
    if (ret != 0)
        return ret;

    // Verifica se este processo pertence ao cgroup do agente
    u32 cgroup_key = 0;
    u64 *agent_cgroup = bpf_map_lookup_elem(&agent_cgroup_map, &cgroup_key);
    if (!agent_cgroup || *agent_cgroup == 0) {
        return 0;   // cgroup não configurado — permite tudo
    }

    u64 current_cgroup = bpf_get_current_cgroup_id();
    if (current_cgroup != *agent_cgroup) {
        return 0;   // processo não é do agente — permite
    }

    // Daqui para frente, só processos do agente LLM são verificados
    // Lê o filename do binário sendo executado (ex: "/usr/bin/perl")
    filename = BPF_CORE_READ(bprm, filename);
    if (!filename)
        return 0;

    bpf_probe_read_kernel_str(path, sizeof(path), filename);

    // Encontra posição da última '/' — loop fixo de BPF_PATH_LEN iters
#pragma unroll
    for (i = 0; i < BPF_PATH_LEN; i++) {
        if (path[i] == '\0') break;
        if (path[i] == '/') slash = i;
    }

    // Copia basename para key.name — loop fixo de MAX_COMMAND_LEN iters
#pragma unroll
    for (i = 0; i < MAX_COMMAND_LEN - 1; i++) {
        int src = slash + 1 + i;
        if (src < 0 || src >= BPF_PATH_LEN) break;
        char c = path[src & (BPF_PATH_LEN - 1)];
        key.name[i] = c;
        if (c == '\0') break;
    }

    blocked = bpf_map_lookup_elem(&blocked_commands, &key);
    if (!blocked)
        return 0;

    // Publica evento no ring buffer antes de bloquear
    event = bpf_ringbuf_reserve(&events, sizeof(*event), 0);
    if (event) {
        pid_tgid = bpf_get_current_pid_tgid();
        event->pid  = pid_tgid >> 32;
        event->tgid = (unsigned int)pid_tgid;
        event->kind = NEMESIS_EVENT_COMMAND_BLOCKED;
        __builtin_memcpy(event->subject, key.name, sizeof(key.name));
        __builtin_memcpy(event->decision, "blocked", 8);
        event->timestamp_ns = bpf_ktime_get_ns();
        bpf_ringbuf_submit(event, 0);
    }

    return -EPERM;
}

#ifndef AF_INET
#define AF_INET 2
#endif
#ifndef AF_INET6
#define AF_INET6 10
#endif

static __always_inline int egress_is_agent(void)
{
    u32 cgroup_key = 0;
    u64 *agent_cgroup = bpf_map_lookup_elem(&agent_cgroup_map, &cgroup_key);
    if (!agent_cgroup || *agent_cgroup == 0)
        return 0;
    return bpf_get_current_cgroup_id() == *agent_cgroup;
}

static __always_inline void egress_emit(const char *label)
{
    struct nemesis_event *event = bpf_ringbuf_reserve(&events, sizeof(*event), 0);
    if (!event)
        return;
    u64 pid_tgid = bpf_get_current_pid_tgid();
    event->pid  = pid_tgid >> 32;
    event->tgid = (unsigned int)pid_tgid;
    event->kind = NEMESIS_EVENT_EGRESS_BLOCKED;
    __builtin_memset(event->subject, 0, sizeof(event->subject));
    bpf_probe_read_kernel_str(event->subject, sizeof(event->subject), label);
    __builtin_memcpy(event->decision, "blocked", 8);
    event->timestamp_ns = bpf_ktime_get_ns();
    bpf_ringbuf_submit(event, 0);
}

SEC("lsm/socket_connect")
int BPF_PROG(nemesis_check_connect, struct socket *sock, struct sockaddr *address,
             int addrlen, int ret)
{
    if (ret != 0)
        return ret;

    // enforce desligado ⇒ não impõe (modo observação)
    u32 zero = 0;
    u8 *enforce = bpf_map_lookup_elem(&egress_enforce, &zero);
    if (!enforce || *enforce == 0)
        return 0;

    if (!egress_is_agent())
        return 0;

    sa_family_t family = BPF_CORE_READ(address, sa_family);

    if (family == AF_INET) {
        struct sockaddr_in *sin = (struct sockaddr_in *)address;
        struct egress_v4_key key = {};
        key.prefixlen = 32;
        __u32 daddr = BPF_CORE_READ(sin, sin_addr.s_addr); // ordem de rede
        __builtin_memcpy(key.addr, &daddr, 4);
        __u16 dport = bpf_ntohs(BPF_CORE_READ(sin, sin_port));

        struct egress_val *val = bpf_map_lookup_elem(&egress_allow_v4, &key);
        if (val && (val->port == 0 || val->port == dport))
            return 0; // destino allowlistado

        egress_emit("ipv4");
        return -EPERM;
    }

    if (family == AF_INET6) {
        struct sockaddr_in6 *sin6 = (struct sockaddr_in6 *)address;
        struct egress_v6_key key = {};
        key.prefixlen = 128;
        BPF_CORE_READ_INTO(&key.addr, sin6, sin6_addr.in6_u.u6_addr8);
        __u16 dport = bpf_ntohs(BPF_CORE_READ(sin6, sin6_port));

        struct egress_val *val = bpf_map_lookup_elem(&egress_allow_v6, &key);
        if (val && (val->port == 0 || val->port == dport))
            return 0;

        egress_emit("ipv6");
        return -EPERM;
    }

    // Famílias não-IP (AF_UNIX, etc.) — não são egress de rede; permite.
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
