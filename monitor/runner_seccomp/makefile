CC = gcc
INCLUDE = /usr/local/include

all: build generate-bpf human-readable

build: runner_seccomp.c
	${CC} runner_seccomp.c -o gen_runner_seccomp -lseccomp -I ${INCLUDE}

generate-bpf: gen_runner_seccomp
	./gen_runner_seccomp > runner_seccomp.bpf

human-readable: gen_runner_seccomp
	./gen_runner_seccomp human

