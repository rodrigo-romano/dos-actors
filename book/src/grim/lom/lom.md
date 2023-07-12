# Linear Optical Model

The linear optical model is a set of optical sensitivity matrices that takes the rigid body motion of M1 and M2 segments as input and outputs optical metrics such as tip-tilt, segment tip-tilt and segment piston .

 * DOS Client

|||||
|-|-|-|-|
| LOM |`gmt_dos-clients_lom`| [crates.io](https://crates.io/crates/gmt_dos-clients_lom) | [docs.rs](https://docs.rs/gmt_dos-clients_lom) | [github](https://github.com/rconan/dos-actors/tree/main/clients/lom) |
 
 * GMT LOM Crate


|||||
|-|-|-|-|
|`gmt-lom`| [crates.io](https://crates.io/crates/gmt-lom) | [docs.rs](https://docs.rs/gmt-lom) | [github](https://github.com/rconan/gmt-lom) |

## [RigidBodyMotionsToLinearOpticalModel](https://docs.rs/gmt_dos-clients_lom/latest/gmt_dos_clients_lom/struct.RigidBodyMotionsToLinearOpticalModel.html) IO 

| Types | Read | Write | Size |
| ----- |:----:|:-----:|:----:|
| `gmt_m1::M1RigidBodyMotions` | `X` | - | - |
| `gmt_m2::M2RigidBodyMotions` | `X` | - | - |
| `optical_metrics::TipTilt` | - | `X` | - |
| `optical_metrics::SegmentTipTilt` | - | `X` | - |
| `optical_metrics::SegmentPiston` | - | `X` | - |