#ifndef NEMESIS_MAPS_H
#define NEMESIS_MAPS_H

#define MAX_COMMAND_LEN 32
#define MAX_SUBJECT_LEN 256
#define MAX_DECISION_LEN 32

enum nemesis_event_kind {
    NEMESIS_EVENT_COMMAND_BLOCKED = 1,
    NEMESIS_EVENT_WRITE_PATH_BLOCKED = 2,
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

#endif
