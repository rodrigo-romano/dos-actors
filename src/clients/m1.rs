/*!
# GMT M1 control clients

The module implements the client interface for the GMT M1 control model from the [m1-ctrl] crate,
it includes:
 - the hardpoints dynamics client
 - the hardpoints load cells client
 - the M1 segment actuators force loop client

The 3 clients are enabled with the `m1-ctrl` feature.

# Example

M1 hardpoints actor:
```
use dos_actors::clients::m1::*;
use dos_actors::prelude::*;
let mut m1_hardpoints: Actor<_> = m1_ctrl::hp_dynamics::Controller::new().into();
```

M1 load cells actor:
```
# use dos_actors::clients::m1::*;
# use dos_actors::prelude::*;
let sim_sampling_frequency: usize = 1000;//Hz
const M1_RATE: usize = 10;
assert_eq!(sim_sampling_frequency / M1_RATE, 100);
let mut m1_hp_loadcells: Actor<_, 1, M1_RATE> =
    m1_ctrl::hp_load_cells::Controller::new().into();

```

M1 segments actuators actors:
```
# use dos_actors::clients::m1::*;
# use dos_actors::prelude::*;
# let sim_sampling_frequency: usize = 1000;//Hz
# const M1_RATE: usize = 10;
# assert_eq!(sim_sampling_frequency / M1_RATE, 100);
let mut m1_segment1: Actor<_, M1_RATE, 1> =
    m1_ctrl::actuators::segment1::Controller::new().into();
let mut m1_segment2: Actor<_, M1_RATE, 1> =
    m1_ctrl::actuators::segment2::Controller::new().into();
let mut m1_segment3: Actor<_, M1_RATE, 1> =
    m1_ctrl::actuators::segment3::Controller::new().into();
let mut m1_segment4: Actor<_, M1_RATE, 1> =
    m1_ctrl::actuators::segment4::Controller::new().into();
let mut m1_segment5: Actor<_, M1_RATE, 1> =
    m1_ctrl::actuators::segment5::Controller::new().into();
let mut m1_segment6: Actor<_, M1_RATE, 1> =
    m1_ctrl::actuators::segment6::Controller::new().into();
let mut m1_segment7: Actor<_, M1_RATE, 1> =
    m1_ctrl::actuators::segment7::Controller::new().into();
```

[m1-ctrl]: https://docs.rs/m1-ctrl/latest/m1_ctrl/
*/

use crate::{
    impl_read, impl_update, impl_write,
    io::{Data, Read, Write},
    Update,
};
#[cfg(feature = "fem")]
use fem::fem_io::{OSSHardpointD, OSSHarpointDeltaF};
use m1_ctrl::{hp_dynamics, hp_load_cells};
use nalgebra as na;
use std::{env, fs::File, ops::Range, path::Path, ptr, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum M1Error {
    #[error("Mode-to-force matrix file not found")]
    Mode2ForceFileNotFound(#[from] std::io::Error),
    #[error("Mode-to-force deserilization failed")]
    Mode2ForceBin(#[from] bincode::Error),
}
pub type Result<T> = std::result::Result<T, M1Error>;

/// hp_dynamics input
pub enum M1RBMcmd {}
/// hp_dynamics output
pub enum HPFcmd {}
/// hp_load_cells input
pub enum M1HPD {}
/// hp_load_cells input
pub enum M1HPcmd {}
/// hp_load_cells output
pub enum M1HPLC {}
/// M1 segment modes
pub enum M1ModalCmd {}

/// Convert M1 modes to actuator forces
pub struct Mode2Force<const S: usize> {
    range: Option<Range<usize>>,
    mode_2_force: na::DMatrix<f64>,
    mode: na::DVector<f64>,
    force: Option<na::DVector<f64>>,
}
impl<const S: usize> Mode2Force<S> {
    /// Creates a new mode 2 forces instance for M1 segment #`S`
    ///
    /// The matrices are loaded from [bincode] files given by `path`.
    /// The root directory is given by the environment variable `M1CALIBRATION`
    /// or is the current directory if `M1CALIBRATION` is not set
    pub fn new<P>(n_actuator: usize, n_mode: usize, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let root_env = env::var("M1CALIBRATION").unwrap_or_else(|_| ".".to_string());
        let root = Path::new(&root_env);
        let mode_2_force = {
            let mode_2_force: Vec<f64> = bincode::deserialize_from(File::open(root.join(path))?)?;
            na::DMatrix::from_vec(n_actuator, n_mode, mode_2_force)
        };
        Ok(Self {
            range: None,
            mode_2_force,
            mode: na::DVector::zeros(n_mode),
            force: None,
        })
    }
    /// Set the expect # of input modes
    pub fn n_input_mode(self, n: usize) -> Self {
        Self {
            range: Some(n * (S - 1)..n * S),
            ..self
        }
    }
}
impl<const S: usize> Update for Mode2Force<S> {
    fn update(&mut self) {
        self.force = Some(&self.mode_2_force * &self.mode);
    }
}
impl<U, const S: usize> Write<Vec<f64>, U> for Mode2Force<S> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, U>>> {
        self.force
            .as_ref()
            .map(|force| Arc::new(Data::new(force.as_slice().to_vec())))
    }
}
impl<const S: usize> Read<Vec<f64>, M1ModalCmd> for Mode2Force<S> {
    fn read(&mut self, data: Arc<Data<Vec<f64>, M1ModalCmd>>) {
        if let Some(range) = &self.range {
            self.mode
                .iter_mut()
                .zip(&(**data)[range.to_owned()])
                .for_each(|(m, d)| *m = *d);
        } else {
            self.mode
                .iter_mut()
                .zip(&(**data))
                .for_each(|(m, d)| *m = *d);
        }
    }
}

impl_update! {hp_dynamics}
impl_read! {hp_dynamics, (M1RBMcmd, m1_rbm_cmd) }
impl_write! {hp_dynamics, (HPFcmd,  hp_f_cmd)}

impl_update! {hp_load_cells}
impl_read! {hp_load_cells, (M1HPD, m1_hp_d), (M1HPcmd, m1_hp_cmd) }
impl_write! {hp_load_cells, (M1HPLC,  m1_hp_lc)}

#[cfg(feature = "fem")]
impl_write! {OSSHarpointDeltaF, hp_dynamics, (HPFcmd,  hp_f_cmd)}
#[cfg(feature = "fem")]
impl_read! {OSSHarpointDeltaF, hp_load_cells, (M1HPcmd, m1_hp_cmd)}
#[cfg(feature = "fem")]
impl_read! {OSSHardpointD, hp_load_cells, (M1HPD, m1_hp_d)}

use paste::paste;
macro_rules! impl_segments {
    ($($sid:expr),+) => {
        $(
            paste! {
		#[doc = "M1 Segment #$sid hardpoint load cells output"]
		pub enum [<S $sid HPLC>] {}
		impl<'a> Write<Vec<f64>, [<S $sid HPLC>]> for hp_load_cells::Controller<'a> {
		    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, [<S $sid HPLC>]>>> {
			let hp_load_cells::Y::M1HPLC(val) = &mut self.m1_hp_lc;
			let mut data = vec![0f64; 6];
			let i: usize = 6*($sid - 1);
			unsafe { ptr::copy_nonoverlapping(val[i..].as_ptr(), data.as_mut_ptr(), 6) }
			Some(Arc::new(Data::new(data)))
		    }
		}
		impl<'a> Update for m1_ctrl::actuators::[<segment $sid>]::Controller<'a> {
		    fn update(&mut self) {
			log::debug!("update");
			self.next();
		    }
		}
		impl<'a> Read<Vec<f64>, [<S $sid HPLC>]> for m1_ctrl::actuators::[<segment $sid>]::Controller<'a> {
		    fn read(&mut self, data: Arc<Data<Vec<f64>, [<S $sid HPLC>]>>) {
			if let m1_ctrl::actuators::[<segment $sid>]::U::HPLC(val) = &mut self.hp_lc {
			    unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
			}
		    }
		}
		pub enum [<S $sid SAoffsetFcmd>] {}
		impl<'a> Read<Vec<f64>, [<S $sid SAoffsetFcmd>]> for m1_ctrl::actuators::[<segment $sid>]::Controller<'a> {
		    fn read(&mut self, data: Arc<Data<Vec<f64>, [<S $sid SAoffsetFcmd>]>>) {
			if let m1_ctrl::actuators::[<segment $sid>]::U::SAoffsetFcmd(val) = &mut self.sa_offsetf_cmd {
			    unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
			}
		    }
		}
		#[cfg(feature = "fem")]
		use fem::fem_io::[<M1ActuatorsSegment $sid>];
		#[cfg(feature = "fem")]
		impl<'a> Write<Vec<f64>, [<M1ActuatorsSegment $sid>]> for m1_ctrl::actuators::[<segment $sid>]::Controller<'a> {
		    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, [<M1ActuatorsSegment $sid>]>>> {
			let m1_ctrl::actuators::[<segment $sid>]::Y::ResActF(val) = &mut self.res_act_f;
			let mut data = vec![0f64; val.len()];
			unsafe { ptr::copy_nonoverlapping(val.as_ptr(), data.as_mut_ptr(), data.len()) }
			Some(Arc::new(Data::new(data)))
		    }
		}
            }
        )+
    };
}
impl_segments! {1,2,3,4,5,6,7}

/*
enum Segment<'a, const N: usize> {
    S1(Actor<actuators::segment1::Controller<'a>, N, 1>),
    S2(Actor<actuators::segment2::Controller<'a>, N, 1>),
    S3(Actor<actuators::segment3::Controller<'a>, N, 1>),
    S4(Actor<actuators::segment4::Controller<'a>, N, 1>),
    S5(Actor<actuators::segment5::Controller<'a>, N, 1>),
    S6(Actor<actuators::segment6::Controller<'a>, N, 1>),
    S7(Actor<actuators::segment7::Controller<'a>, N, 1>),
}
type D = Vec<f64>;
impl<const N: usize> Segment<'static, N> {
    pub fn load_cells_channel(
        &mut self,
        load_cells: &mut Actor<hp_load_cells::Controller<'static>, 1, N>,
        cap: Option<usize>,
    ) {
        use Segment::*;
        let vcap = cap.map(|x| vec![x]);
        match self {
            S1(segment) => {
                load_cells.add_output::<D, S1HPLC>(vcap).into_input(segment);
            }
            S2(segment) => {
                load_cells.add_output::<D, S2HPLC>(vcap).into_input(segment);
            }
            S3(segment) => {
                load_cells.add_output::<D, S3HPLC>(vcap).into_input(segment);
            }
            S4(segment) => {
                load_cells.add_output::<D, S4HPLC>(vcap).into_input(segment);
            }
            S5(segment) => {
                load_cells.add_output::<D, S5HPLC>(vcap).into_input(segment);
            }
            S6(segment) => {
                load_cells.add_output::<D, S6HPLC>(vcap).into_input(segment);
            }
            S7(segment) => {
                load_cells.add_output::<D, S7HPLC>(vcap).into_input(segment);
            }
        };
    }
    pub fn fem_channel<T>(
        &mut self,
        fem: &mut Actor<DiscreteModalSolver<T>, 1, 1>,
        cap: Option<usize>,
    ) where
        T: 'static + Solver + Send + Default,
        DiscreteModalSolver<T>: Iterator,
    {
        use Segment::*;
        let vcap = cap.map(|x| vec![x]);
        match self {
            S1(segment) => {
                segment
                    .add_output::<D, M1ActuatorsSegment1>(vcap)
                    .into_input(fem);
            }
            S2(segment) => {
                segment
                    .add_output::<D, M1ActuatorsSegment2>(vcap)
                    .into_input(fem);
            }
            S3(segment) => {
                segment
                    .add_output::<D, M1ActuatorsSegment3>(vcap)
                    .into_input(fem);
            }
            S4(segment) => {
                segment
                    .add_output::<D, M1ActuatorsSegment4>(vcap)
                    .into_input(fem);
            }
            S5(segment) => {
                segment
                    .add_output::<D, M1ActuatorsSegment5>(vcap)
                    .into_input(fem);
            }
            S6(segment) => {
                segment
                    .add_output::<D, M1ActuatorsSegment6>(vcap)
                    .into_input(fem);
            }
            S7(segment) => {
                segment
                    .add_output::<D, M1ActuatorsSegment7>(vcap)
                    .into_input(fem);
            }
        }
    }
    pub fn boxed(&mut self) -> Box<&mut dyn Run> {
        use Segment::*;
        match self {
            S1(segment) => Box::new(segment),
            S2(segment) => Box::new(segment),
            S3(segment) => Box::new(segment),
            S4(segment) => Box::new(segment),
            S5(segment) => Box::new(segment),
            S6(segment) => Box::new(segment),
            S7(segment) => Box::new(segment),
        }
    }
    pub async fn bootstrap(&mut self) -> Result<()> {
        use Segment::*;
        match self {
            S1(segment) => segment.async_bootstrap::<D, M1ActuatorsSegment1>().await,
            S2(segment) => segment.async_bootstrap::<D, M1ActuatorsSegment2>().await,
            S3(segment) => segment.async_bootstrap::<D, M1ActuatorsSegment3>().await,
            S4(segment) => segment.async_bootstrap::<D, M1ActuatorsSegment4>().await,
            S5(segment) => segment.async_bootstrap::<D, M1ActuatorsSegment5>().await,
            S6(segment) => segment.async_bootstrap::<D, M1ActuatorsSegment6>().await,
            S7(segment) => segment.async_bootstrap::<D, M1ActuatorsSegment7>().await,
        }
    }
}
impl<'a, const N: usize> Display for Segment<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Segment::*;
        match self {
            S1(segment) => segment.fmt(f),
            S2(segment) => segment.fmt(f),
            S3(segment) => segment.fmt(f),
            S4(segment) => segment.fmt(f),
            S5(segment) => segment.fmt(f),
            S6(segment) => segment.fmt(f),
            S7(segment) => segment.fmt(f),
        }
    }
}
pub struct M1Builder<'a, const N: usize> {
    segments: Vec<Segment<'a, N>>,
}
impl<const N: usize> M1Builder<'static, N> {
    pub fn new() -> Self {
        Self {
            segments: vec![
                Segment::S1(actuators::segment1::Controller::new().into()),
                Segment::S2(actuators::segment2::Controller::new().into()),
                Segment::S3(actuators::segment3::Controller::new().into()),
                Segment::S4(actuators::segment4::Controller::new().into()),
                Segment::S5(actuators::segment5::Controller::new().into()),
                Segment::S6(actuators::segment6::Controller::new().into()),
                Segment::S7(actuators::segment7::Controller::new().into()),
            ],
        }
    }
    pub fn keep(self, keep: [bool; 7]) -> Self {
        let mut segments = self.segments;
        let mut iter = keep.iter();
        segments.retain(|_| *iter.next().unwrap());
        Self { segments, ..self }
    }
    pub fn build<T>(self, fem: &mut Actor<DiscreteModalSolver<T>, 1, 1>) -> M1<'static, N>
    where
        T: 'static + Solver + Send + Default,
        DiscreteModalSolver<T>: Iterator,
    {
        let mut load_cells: Actor<_, 1, N> = hp_load_cells::Controller::new().into();
        fem.add_output::<D, OSSHardpointD>(None)
            .into_input(&mut load_cells);
        let mut hardpoints: Actor<_, 1, 1> = hp_dynamics::Controller::new().into();
        hardpoints
            .add_output::<D, OSSHarpointDeltaF>(Some(vec![1, 1]))
            .into_input(fem)
            .into_input(&mut load_cells);
        let mut segments = self.segments;
        for segment in &mut segments {
            segment.load_cells_channel(&mut load_cells, None);
            segment.fem_channel(fem, None);
        }
        M1 {
            hardpoints,
            load_cells,
            segments,
        }
    }
}
pub struct M1<'a, const N: usize> {
    pub hardpoints: Actor<hp_dynamics::Controller<'a>, 1, 1>,
    pub load_cells: Actor<hp_load_cells::Controller<'a>, 1, N>,
    segments: Vec<Segment<'a, N>>,
}
impl<const N: usize> M1<'static, N> {
    pub fn builder() -> M1Builder<'static, N> {
        log::debug!("M1 building!");
        M1Builder::new()
    }
    /// Bootstraps the segments actuator forces
    pub async fn bootstrap(&mut self) -> Result<()> {
        log::debug!("M1 bootstrapping segments!");
        for segment in &mut self.segments {
            segment.bootstrap().await?;
        }
        log::debug!("M1 segments bootstrapped!");
        Ok(())
    }
}
impl<'a, const N: usize> Display for M1<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hardpoints.fmt(f)?;
        self.load_cells.fmt(f)?;
        for segment in &self.segments {
            segment.fmt(f)?;
        }
        Ok(())
    }
}
#[async_trait]
impl<const N: usize> Run for M1<'static, N> {
    async fn async_run(&mut self) -> Result<()> {
        let mut hardware: Vec<Box<&mut dyn Run>> = vec![
            Box::new(&mut self.hardpoints),
            Box::new(&mut self.load_cells),
        ];
        for segment in &mut self.segments {
            hardware.push(segment.boxed());
        }
        log::debug!("M1 joining futures!");
        let futures: Vec<_> = hardware.into_iter().map(|h| h.async_run()).collect();
        join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }
}
*/
/*
pub mod assembly {
    use crate::{one_to_many, print_error, Actor, Client};
    pub struct Controller<I, O, const NI: usize, const NO: usize>
    where
        I: Default + std::fmt::Debug,
        O: Default + std::fmt::Debug,
    {
        sid: Vec<usize>,
        segment: Vec<Actor<I, O, NI, NO>>,
    }
    impl<I, O, const NI: usize, const NO: usize> Controller<I, O, NI, NO>
    where
        I: Default + std::fmt::Debug,
        O: Default + std::fmt::Debug + Clone,
        Vec<O>: Clone,
    {
        pub fn new<T, U, V, const L: usize, const B: usize, const F: usize>(
            loadcells: &mut Actor<T, I, L, NI>,
            bending_modes: &mut [Actor<U, I, B, NI>],
            fem: &mut Actor<O, V, NO, F>,
        ) -> Self
        where
            T: Default + std::fmt::Debug,
            U: Default + std::fmt::Debug,
            V: Default + std::fmt::Debug,
        {
            let mut segment: Vec<_> = (1..=7)
                .map(|sid| Actor::<I, O, NI, NO>::new().tag(format!("M1 S{sid}")))
                .collect();
            one_to_many(
                loadcells,
                &mut segment
                    .iter_mut()
                    .collect::<Vec<&mut Actor<I, O, NI, NO>>>()
                    .as_mut_slice(),
            );
            segment
                .iter_mut()
                .zip(bending_modes.iter_mut())
                .for_each(|(si, bmi)| {
                    one_to_many(bmi, &mut [si]);
                    one_to_many(si, &mut [fem]);
                });
            Self {
                sid: vec![1, 2, 3, 4, 5, 6, 7],
                segment,
            }
        }
    }
    impl<const NI: usize, const NO: usize> Controller<Vec<f64>, Vec<f64>, NI, NO> {
        pub fn spawn(self) {
            async fn spawn_a_segment<const NI: usize, const NO: usize>(
                mut si: Actor<Vec<f64>, Vec<f64>, NI, NO>,
                data: Vec<Vec<f64>>,
                client: &mut impl Client<I = Vec<f64>, O = Vec<f64>>,
            ) {
                if let Err(e) = si.bootstrap(Some(data)).await {
                    print_error(format!("{} distribute ended", si.tag.as_ref().unwrap()), &e);
                }
                if let Err(e) = si.run(client).await {
                    print_error(format!("{} loop ended", si.tag.as_ref().unwrap()), &e);
                };
            }
            for (sid, si) in self.sid.into_iter().zip(self.segment.into_iter()) {
                match sid {
                    1 => {
                        tokio::spawn(async move {
                            spawn_a_segment(
                                si,
                                vec![vec![0f64; 335]],
                                &mut m1_ctrl::actuators::segment1::Controller::new(),
                            )
                            .await;
                        });
                    }
                    2 => {
                        tokio::spawn(async move {
                            spawn_a_segment(
                                si,
                                vec![vec![0f64; 335]],
                                &mut m1_ctrl::actuators::segment2::Controller::new(),
                            )
                            .await;
                        });
                    }
                    3 => {
                        tokio::spawn(async move {
                            spawn_a_segment(
                                si,
                                vec![vec![0f64; 335]],
                                &mut m1_ctrl::actuators::segment3::Controller::new(),
                            )
                            .await;
                        });
                    }
                    4 => {
                        tokio::spawn(async move {
                            spawn_a_segment(
                                si,
                                vec![vec![0f64; 335]],
                                &mut m1_ctrl::actuators::segment4::Controller::new(),
                            )
                            .await;
                        });
                    }
                    5 => {
                        tokio::spawn(async move {
                            spawn_a_segment(
                                si,
                                vec![vec![0f64; 335]],
                                &mut m1_ctrl::actuators::segment5::Controller::new(),
                            )
                            .await;
                        });
                    }
                    6 => {
                        tokio::spawn(async move {
                            spawn_a_segment(
                                si,
                                vec![vec![0f64; 335]],
                                &mut m1_ctrl::actuators::segment6::Controller::new(),
                            )
                            .await;
                        });
                    }
                    7 => {
                        tokio::spawn(async move {
                            spawn_a_segment(
                                si,
                                vec![vec![0f64; 306]],
                                &mut m1_ctrl::actuators::segment7::Controller::new(),
                            )
                            .await;
                        });
                    }
                    _ => panic!("invalid segment #"),
                }
            }
        }
    }
}
     */
