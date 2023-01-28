# Multirate System

A multirate system mixes signals sampled at different frame rates.
The flowchart below is a depiction of such a system implemented with `gmt_dos-actors` where the sample rate of the inputs and outputs (IO) is color coded according to the following table of sample rates:

| green | orange | purple |
|:-----:|:------:|:------:|
| 1 | 4 | 2 |

![Multirate model](multirate-model.dot.svg)

The purple IO and orange IO are, respectively, 1/2 and 1/4 the sampling rate of the green IO as show in the outputs record:

![Multirate logs](multirate_out.png)

An actor with an output rate (`NO`) greater than the input rate (`NI`) (i.e. `NO>NI`), downsamples the outputs according to the ratio `NI:NO`.

An actor with an input rate (`NI`) greater than the input rate (`NO`) (i.e. `NI>NO`), upsamples the outputs with a zero-order-hold, the zero-order-hold is `NI/NO` sample long.

In both cases, downsampling and upsampling,
the `Update`method of the actor's client is invoked at the input rate.

The `gmt_dos-actors` implementation of the multirate system above starts by setting the downsampling and upsampling rates:
```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:rates}}
```

The input signal is a ramp starting a 0 with unitary step increments:
```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:signal}}
```

The [Sampler](https://docs.rs/gmt_dos-actors/latest/gmt_dos_actors/clients/struct.Sampler.html) client is used for rate transition:
 * downsampling from 1 to 4
 ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:downsampling}}
```
* upsampling from 4 to 2
```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:upsampling}}
```

Downsampling is also the results of the [Average](https://docs.rs/gmt_dos-actors/latest/gmt_dos_actors/clients/struct.Average.html) client which averages the input over `NO/NI` samples:
```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:average}}
```

The downsampled and averaged signals, both with the same sampling rate, are recombined with the `SignedDiff` client which computes the difference between both signals and alternates the output sign:
```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:signed_diff}}
```

In the next step, we define 3 loggers, one for each sampling rate:
 * 1
 ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:logging}}
```
 * 4
  ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:downlogging}}
```
 * 2
  ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:uplogging}}
```

Then it's a matter of defining inputs and outputs:
  ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:io}}
```
 building the network:
  ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:network}}
```
and running the model:
  ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:model}}
```

Finally, the logged ouputs are synchronized by post-proccessing the saved data while remembering that if the sampling rate of the ramp signal is 1 and its time step is `i`, then the time step of the downsampled and upsampled signals are derived from `DOWNRATE * (i + 1) - 1` and `UPRATE * i + DOWNRATE - 1`, respectively.
  ```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:log}}
```

#### Implementation of the `SignedDiff` client:
```rust,no_run,noplayground
{{#include ../../examples/multirate.rs:sdiff_client}}
```