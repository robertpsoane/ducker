
# Images
- Run Image
- New Image
- Tag Image

# Exec
- Add modal to provide exec command when exec-ing into a container

# Logs
- Choose log time range

# In Depth View
- Ability to see description of a resource in depth

# Filesystems
- Access image/container filesystem & get file

# Other/Tech Debt
- Fix error message when docker isn't installed/docker.sock is unavailable
    - Potentially offer alternative sock path as well?
- Proper help page in repo & included in app somewhere
- Use mod.rs; current setup is hella confusing


- Modals to use trait objects
- Fix callbacks to use closures
- Add filter widget/component (similar to vim searching)
- General purpose modals for errors
- Add tracing of some sort
- Simplify page loading/closing lifecycle
    - Can visibility be removed, and just rely on constructor?

# Not currently in scope
- Swarm specific features
