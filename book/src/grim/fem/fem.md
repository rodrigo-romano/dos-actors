# Finite Element Model

The GMT finite element model is loaded from the zip file: `modal_state_space_model_2ndOrder.zip`.
The path to the zip file must be affected to the environment variable: `FEM_REPO`.
The zip file is created with the Matlab script [unwrapFEM](https://github.com/rconan/fem/blob/main/tools/unwrapFEM.m) using data produced by the GMT Integrated Modeling team.

The FEM model is stored into the `gmt-fem` crate as a continuous second order ODE and the `gmt_dos-clients_fem` crate transforms the FEM into discrete 2x2 state space models with as many model as the number of eigen modes of the FEM.

 * DOS Client

|||||
|-|-|-|-|
|`gmt_dos-clients_fem`| [crates.io](https://crates.io/crates/gmt_dos-clients_fem) | [docs.rs](https://docs.rs/gmt_dos-clients_fem) | [github](https://github.com/rconan/dos-actors/tree/main/clients/fem) |
 
 * GMT FEM Crate


|||||
|-|-|-|-|
|`gmt-fem`| [crates.io](https://crates.io/crates/gmt-fem) | [docs.rs](https://docs.rs/gmt-fem) | [github](https://github.com/rconan/fem) |


## `DiscreteModalSolver` IO 

| Types | Read | Write | Size |
| ----- |:----:|:-----:|:----:|
| `mount::MountEncoders` | - | `X` | - |
| `mount::MountTorques` | `X` | - | - |
| `gmt_m1::M1RigidBodyMotions` | - | `X` | `42` |
| `gmt_m1::M1ModeShapes` | - | `X` | - |
| `gmt_m1::segment::ActuatorAppliedForces<ID>` | `X` | - | - |
| `gmt_m1::segment::HardpointsForces<ID>` | `X` | - | - |
| `gmt_m1::segment::HardpointsMotion<ID>` | - | `X` | - | 
| `gmt_m1::segment::RBM<ID>` | - | `X` | - | 
| `gmt_m2::M2RigidBodyMotions` | - | `X` | `42` |
| `gmt_m2::M2PositionerForces` | `X` | - | - |
| `gmt_m2::M2PositionerNodes` | - | `X` | - |
| `gmt_m2::M2FSMPiezoForces` | `X` | - | - |
| `gmt_m2::M2FSMPiezoNodes` | - | `X` | - |
| `gmt_m2::asm::M2ASMColdPlateForces` | `X` | - | - |
| `gmt_m2::asm::M2ASMFaceSheetForces` | `X` | - | - |
| `gmt_m2::asm::M2ASMFaceSheetNodes` | - | `X` | - |
| `gmt_m2::asm::M2ASMRigidBodyForces` | `X` | - | - |
| `gmt_m2::asm::M2ASMRigidBodyNodes` | - | `X` | - |
| `gmt_m2::asm::segment::VoiceCoilsForces<ID>` | `X` | - | `675` |
| `gmt_m2::asm::segment::VoiceCoilsMotion<ID>` | - | `X` | `675` |
| `gmt_m2::asm::segment::FluidDampingForces<ID>` | `X` | - | `675` |
| `gmt_m2::asm::segment::FaceSheetFigure<ID>` | - | `X` | - |
| `cfd_wind_loads::CFDMountWindLoads` | `X` | - | - |
| `cfd_wind_loads::CFDM1WindLoads` | `X` | - | - |
| `cfd_wind_loads::CFDM2WindLoads` | `X` | - | - |