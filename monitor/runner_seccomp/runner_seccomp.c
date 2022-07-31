#include <seccomp.h>
#include <stdio.h>
#include <stdbool.h>
#include <string.h>

int main(int argc, char *argv[])
{
    bool human_readable = false;

    // Check if user wants human readable form or bpf code
    switch (argc)
    {
    case 1:
        break;
    case 2:
        if (strcmp(argv[1], "human") == 0)
        {
            human_readable = true;
            break;
        }

        printf("Argument '%s' is invalid in this context.\n\nIf you'd like to display the generated filter in human readable form, please use the 'human' argument.\n", argv[1]);
        return -1;
    default:
        printf("Tool does not allow more than one argument.\n\nIf you'd like to display the generated filter in human readable form, please use the 'human' argument.\n");
        return -1;
    }

    scmp_filter_ctx ctx = seccomp_init(SCMP_ACT_KILL_PROCESS);

    if (!ctx)
    {
        printf("Failed to initialize seccomp filter context\n");
        return -1;
    }

    int rc = seccomp_arch_add(ctx, SCMP_ARCH_AARCH64);
    if (rc < 0)
        goto err;

    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(futex), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(ppoll), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(epoll_pwait), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(ioctl), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(openat), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(close), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(write), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(timerfd_settime), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(fstat), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(clock_nanosleep), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(sched_yield), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(read), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(getrandom), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(faccessat), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(readlinkat), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(mprotect), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(getdents64), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(getcwd), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(clone), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(recvmsg), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(mmap), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(uname), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(munmap), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(newfstatat), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(eventfd2), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(setsockopt), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(sigaltstack), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(timerfd_create), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(madvise), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(socket), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(set_robust_list), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(recvfrom), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(brk), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(bind), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(rt_sigaction), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(fcntl), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(epoll_ctl), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(sched_getaffinity), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(statx), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(connect), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(getsockname), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(prctl), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(epoll_create1), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(prlimit64), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(mkdirat), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(shutdown), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(statfs), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(getsockopt), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(gettid), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(lseek), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(rt_sigprocmask), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(getpid), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(set_tid_address), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(mremap), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(execve), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(wait4), 0);
    if (rc < 0)
        goto err;
    rc = seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(exit), 0);
    if (rc < 0)
        goto err;

    if (human_readable)
    {
        seccomp_export_pfc(ctx, 1);
    }
    else
    {
        seccomp_export_bpf(ctx, 1);
    }

    seccomp_release(ctx);
    return 0;

err:
    printf("Failed to setup a seccomp rule. This might be caused if the same rule is registered twice.\n");
    return -1;
}