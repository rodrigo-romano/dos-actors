# M2 ASMS Edge Sensors Feed-Forward and Shell Off-load

The M2 ASMS edge sensors model is run within the `dos-actors/clients/servos/edge-sensors` folder with
```shell
. setup.sh 
cargo run --release --bin m2
```
It relies solely on the FEM static gain to transform
some FEM outputs into other FEM outputs.
A description of these transforms is given in the documentation of the`edge-sensor` crate:
```shell
. setup.sh 
cargo doc --no-deps --open
```
Use the documentation to generate the files that contains the necessary transforms.

The scopes of the model are run with
```shell
 cd scopes;cargo run --release
 ```