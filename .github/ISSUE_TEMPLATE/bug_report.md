---
name: Bug report
about: Create a report to help us improve Hyprshell
title: "[Bug]: "
---

## Describe the bug

A clear and concise description of what the bug is.

## To reproduce

Steps to reproduce the behavior:

1. ...
2. ...
3. ...

## Expected behavior

A clear and concise description of what you expected to happen.

## Logs

Please attach logs covering the time the issue occurred. If you start Hyprshell with the systemd user service, collect its recent logs with:

```bash
journalctl --user -u hyprshell.service --since "10 minutes ago" --no-pager > hyprshell.log
```

Adjust the time window as needed, then attach `hyprshell.log` to this issue. 

**The systemd service logs most likely won't include debug/trace-level output.** To capture detailed logs, stop the currently running instance first. For the systemd user service, run:

```bash
systemctl --user stop hyprshell.service
```

Then launch Hyprshell from a terminal:

```bash
hyprshell -vv run 2>&1 | tee hyprshell.log
```

Reproduce the issue, then press `Ctrl+C` in that terminal and attach `hyprshell.log`. The `-vv` option enables trace-level logging. If you stopped the systemd service above, you can restart it afterward with `systemctl --user start hyprshell.service`. If you start Hyprshell another way, stop that instance before running the terminal command.

Please review logs for private information (such as file paths, or window titles) and redact anything you do not want to share before attaching them.

## Debug info

Please run the following command and paste its complete output below:

```bash
hyprshell debug info -vv
```

```text
Paste output here
```

## Screenshots or additional context

If applicable, add screenshots or any other context that helps explain the problem.
