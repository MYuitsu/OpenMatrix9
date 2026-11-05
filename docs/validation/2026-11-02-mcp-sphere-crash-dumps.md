# MCP Sphere GUI crash dump investigation

2026-11-02, Windows x64. Read-only investigation of two Windows LocalCrashDumps:
PID 6152 at 16:51 and PID 23964 at 16:59. Both record exception `0xc000041d`
at `om9!__chkstk+0x37` on the GUI thread. Each executable timestamp matches
the corresponding local debug binary, and cdb loaded its private PDB; cdb
warned that the executable checksum could not be verified. These are minidumps,
not complete memory recordings.

Both stacks show `LiveHost::service` -> MCP `Server::command` ->
`Controller::submit_effect` -> `submit_solid_pending` ->
`submit_solid_sphere_text` -> `accept_sphere_point` -> `__chkstk`. The later
binary's disassembly reserves 0x17480 bytes in `submit_solid_pending`, 0x62e30
in `submit_solid_sphere_text`, and attempts 0x1f8a0 in `accept_sphere_point`.
Its thread stack base is 0x...76100000 and limit 0x...76001000; the failing
stack probe is at 0x...76006000. Together these support stack exhaustion in
the debug GUI build during Sphere point input through MCP. The recorded
exception is the Windows callback exception, not itself a standalone proof of
the original underlying exception code. The dumps do not establish the exact
input, Sphere mode, or whether other builds and invocation paths are safe.

Separately, a Box point request observed a connection reset while the GUI
remained responsive; no new crash dump was found. It is not established to
share the Sphere failure. No application change or crash fix was tested.
Next: reduce stack use on the command path, then reproduce and validate Sphere
through the actual GUI MCP and normal CMD/mouse paths, including Undo and the
previous connection-reset scenario, with a matching debug build.
