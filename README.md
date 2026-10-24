# Squirtle

<img src=".assets/squirtle.gif" alt="A blue sea turtle with sunglasses" />

> [!WARNING]
> Squirtle is not yet in a working state, do not try to use it for yourself.

It's a little-too-big-for-the-beginner-I-am project I started because there's literally **0** fan-control software on Linux that work with my laptop (because apparently nobody figured out how to correctly tame the Dell G-Series devices).

I want to have something cool that'll do the job for me but also for anyone else.

### squirtlectl

The CLI tool that let you control both Squirtle's main daemon (SquirtleD) and the fan-control daemon (Pidgey).

| Command Argument(s) | Description |
|---------------------|:-----------:|
| **daemon** | ---- |
| start | Start SquirtleD |
| stop | Tell SquirtleD to stop |
| **fan-control** | ---- |
| start | Start Pidgey |
| start --standalone | Start Pidgey but independently from SquirtleD |

### squirtle-core

It's the backend of the whole Squirtle project.

Its role is finding & every useful information (HWMON sensors, temperatures, fan states, etc…) and performing every asked operations.

For now, I'm only focussing on cooling-related features, but the whole point of squirtle-core is being able to manipulate the cooling devices, but also some CPU/GPU features, speed, power-limit, ect…

### squirtle-d

The daemon from which a client can plug itself (via UNIX socket) to perform any operation via IPC.

It is here so we can have both Pidgey (or any other thermal-control software based on Squirtle) doing its job and any other client (e.g.: a GNOME extension, or a Plasmoid) showing the relevant information to the user in the same time, without the risk of any concurrency-related problem.

### squirtle-pidgey

Squirtle's multithreaded fan-control daemon.

Its goal is being able to asynchronously keep an eye to the relevant sensors and adjusting the fan speed (as a whole package, or adjusting each fans individually according to user-settings).

The defined temperature/fan-speed curve is step-based by default, but Pidgey will also support linear interpolation.

Pidgey is also being designed to be used with either modes:
- **Client mode (default):** Pidgey will work directly with SquirtleD to perform any operation.
- **Standalone mode:** Pidgey will work only with Squirtle-Core to perform any operation *(useful if Pidgey is the only use you would have of Squirtle, and you're not planning to plug any other client on it (whether it's a GUI, a GNOME extension, a Plasmoid, ect…))*
