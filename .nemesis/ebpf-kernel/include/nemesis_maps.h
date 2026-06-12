#ifndef NEMESIS_MAPS_H
#define NEMESIS_MAPS_H

#define MAX_COMMAND_LEN 32
#define MAX_SUBJECT_LEN 256
#define MAX_DECISION_LEN 32

enum nemesis_event_kind {
    NEMESIS_EVENT_COMMAND_BLOCKED = 1,
    NEMESIS_EVENT_WRITE_PATH_BLOCKED = 2,
    NEMESIS_EVENT_EGRESS_BLOCKED = 3,
};

struct nemesis_event {
    unsigned int pid;
    unsigned int tgid;
    unsigned int kind;
    char subject[MAX_SUBJECT_LEN];
    char decision[MAX_DECISION_LEN];
    unsigned long long timestamp_ns;
};

struct command_key {
    char name[MAX_COMMAND_LEN];
};

/* Chave do LPM trie IPv4: prefixlen (em bits) + 4 bytes de IP em ordem de rede. */
struct egress_v4_key {
    unsigned int prefixlen;
    unsigned char addr[4];
};

/* Chave do LPM trie IPv6: prefixlen + 16 bytes de IP. */
struct egress_v6_key {
    unsigned int prefixlen;
    unsigned char addr[16];
};

/* Valor de ambos os tries: porta permitida em ordem de host (0 = qualquer). */
struct egress_val {
    unsigned short port;
};

#endif
