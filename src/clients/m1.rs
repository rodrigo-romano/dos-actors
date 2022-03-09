//! GMT M1 control interface
//!
//! Interface for GMT [m1-ctrl] model,
//! it includes:
//!  - the hardpoints dynamics
//!  - the hardpoints load cells
//!  - the M1 segment actuators force loop
//!
//! [m1-ctrl](https://docs.rs/m1-ctrl/latest/m1_ctrl/)

/*
macro_rules! impl_read {
    ($name:ty,$val:ident) => {
    pub enum M1RBMcmd {}
    impl<'a> Read<Vec<f64>, M1RBMcmd> for hp_dynamics::Controller<'a> {
        fn read(&mut self, data: Arc<Data<Vec<f64>, M1RBMcmd>>) {
            if let controller::U::M1RBMcmd(val) = &mut self.control.m1_rbm_cmd {
                assert_eq!(
                    data.len(),
                    val.len(),
                    "data size ({}) do not match M1RBMcmd size ({})",
                    data.len(),
                    val.len()
                );
                unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
            }
        }
    }
    };
}*/

use crate::{
    io::{Data, Read, Write},
    Update,
};
use m1_ctrl::{hp_dynamics, hp_load_cells};
use std::{ptr, sync::Arc};

macro_rules! impl_update {
    ($module:ident) => {
        impl<'a> Update for $module::Controller<'a> {
            fn update(&mut self) {
                log::debug!("update");
                self.next();
            }
        }
    };
}
macro_rules! impl_read {
    ($module:ident, ($var:ident, $val:ident)) => {
        #[doc = "$module $var input"]
        pub enum $var {}
        impl<'a> Read<Vec<f64>, $var> for $module::Controller<'a> {
            fn read(&mut self, data: Arc<Data<Vec<f64>, $var>>) {
                let $module::U::$var(val) = &mut self.$val;
                assert_eq!(
                    data.len(),
                    val.len(),
                    "data size ({}) do not match $ident size ({})",
                    data.len(),
                    val.len()
                );
                unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
            }
        }
    };
    ($module:ident, ($var:ident, $val:ident), $(($varo:ident, $valo:ident)),+) => {
        #[doc = "$module $var input"]
        pub enum $var {}
        impl<'a> Read<Vec<f64>, $var> for $module::Controller<'a> {
            fn read(&mut self, data: Arc<Data<Vec<f64>, $var>>) {
                if let $module::U::$var(val) = &mut self.$val {
                    assert_eq!(
                        data.len(),
                        val.len(),
                        "data size ({}) do not match $ident size ({})",
                        data.len(),
                        val.len()
                    );
                    unsafe {
                        ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len())
                    }
                }
            }
        }
	$(
        #[doc = "$module $varo input"]
        pub enum $varo {}
        impl<'a> Read<Vec<f64>, $varo> for $module::Controller<'a> {
            fn read(&mut self, data: Arc<Data<Vec<f64>, $varo>>) {
                if let $module::U::$varo(val) = &mut self.$valo {
                    assert_eq!(
                        data.len(),
                        val.len(),
                        "data size ({}) do not match $ident size ({})",
                        data.len(),
                        val.len()
                    );
                    unsafe {
                        ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len())
                    }
                }
            }
        }
	)+
    };
}
macro_rules! impl_write {
    ($module:ident, $var:ident, $val:ident) => {
        #[doc = "$module $var output"]
        pub enum $var {}
        impl<'a> Write<Vec<f64>, $var> for $module::Controller<'a> {
            fn write(&mut self) -> Option<Arc<Data<Vec<f64>, $var>>> {
                let $module::Y::$var(val) = &mut self.$val;
                let mut data = vec![0f64; val.len()];
                unsafe { ptr::copy_nonoverlapping(val.as_ptr(), data.as_mut_ptr(), data.len()) }
                Some(Arc::new(Data::new(data)))
            }
        }
    };
}

impl_update! {hp_dynamics}
impl_read! {hp_dynamics, (M1RBMcmd, m1_rbm_cmd) }
impl_write! {hp_dynamics, HPFcmd,  hp_f_cmd}

impl_update! {hp_load_cells}
impl_read! {hp_load_cells, (M1HPD, m1_hp_d), (M1HPcmd, m1_hp_cmd) }
impl_write! {hp_load_cells, M1HPLC,  m1_hp_lc}

#[cfg(feature = "fem")]
use fem::fem_io::{OSSHardpointD, OSSHarpointDeltaF};
#[cfg(feature = "fem")]
impl<'a> Write<Vec<f64>, OSSHarpointDeltaF> for hp_dynamics::Controller<'a> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, OSSHarpointDeltaF>>> {
        let hp_dynamics::Y::HPFcmd(val) = &mut self.hp_f_cmd;
        let mut data = vec![0f64; val.len()];
        unsafe { ptr::copy_nonoverlapping(val.as_ptr(), data.as_mut_ptr(), data.len()) }
        Some(Arc::new(Data::new(data)))
    }
}
#[cfg(feature = "fem")]
impl<'a> Read<Vec<f64>, OSSHarpointDeltaF> for hp_load_cells::Controller<'a> {
    fn read(&mut self, data: Arc<Data<Vec<f64>, OSSHarpointDeltaF>>) {
        if let hp_load_cells::U::M1HPcmd(val) = &mut self.m1_hp_cmd {
            assert_eq!(
                data.len(),
                val.len(),
                "data size ({}) do not match $ident size ({})",
                data.len(),
                val.len()
            );
            unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
        }
    }
}
#[cfg(feature = "fem")]
impl<'a> Read<Vec<f64>, OSSHardpointD> for hp_load_cells::Controller<'a> {
    fn read(&mut self, data: Arc<Data<Vec<f64>, OSSHardpointD>>) {
        if let hp_load_cells::U::M1HPD(val) = &mut self.m1_hp_d {
            assert_eq!(
                data.len(),
                val.len(),
                "data size ({}) do not match $ident size ({})",
                data.len(),
                val.len()
            );
            unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
        }
    }
}

use paste::paste;
macro_rules! impl_client_for_segments {
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
impl_client_for_segments! {1,2,3,4,5,6,7}

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
