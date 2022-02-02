//! GMT M1 control model

pub mod hardpoints {
    use crate::Client;
    use m1_ctrl::hp_dynamics;
    impl<'a> Client for hp_dynamics::Controller<'a> {
        type I = Vec<f64>;
        type O = Vec<f64>;
        fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
            log::debug!(
                "receive #{} inputs: {:?}",
                data.len(),
                data.iter().map(|x| x.len()).collect::<Vec<usize>>()
            );
            for (k, v) in data[0].iter().enumerate() {
                self.m1_rbm_cmd[k] = *v;
            }
            self
        }
        fn produce(&mut self) -> Option<Vec<Self::O>> {
            log::debug!("produce");
            Some(vec![(&self.hp_f_cmd).into()])
        }
        fn update(&mut self) -> &mut Self {
            log::debug!("update");
            self.next();
            self
        }
    }
}

pub mod loadcells {
    use crate::Client;
    use m1_ctrl::hp_load_cells;
    impl<'a> Client for hp_load_cells::Controller<'a> {
        type I = Vec<f64>;
        type O = Vec<f64>;
        fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
            log::debug!(
                "receive #{} inputs: {:?}",
                data.len(),
                data.iter().map(|x| x.len()).collect::<Vec<usize>>()
            );
            for (k, v) in data[0].iter().enumerate() {
                self.m1_hp_d[k] = *v;
            }
            for (k, v) in data[1].iter().enumerate() {
                self.m1_hp_cmd[k] = *v;
            }
            self
        }
        fn produce(&mut self) -> Option<Vec<Self::O>> {
            log::debug!("produce");
            Some(vec![(&self.m1_hp_lc).into()])
        }
        fn update(&mut self) -> &mut Self {
            log::debug!("update");
            self.next();
            self
        }
    }
}

pub mod segments {
    use crate::Client;
    use m1_ctrl::actuators;
    use paste::paste;
    macro_rules! impl_client_for_segments {
	($($sid:expr),+) => {
	    $(
		paste! {
		    impl<'a> Client for actuators::[<segment $sid>]::Controller<'a> {
			type I = Vec<f64>;
			type O = Vec<f64>;
			fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
			    log::debug!(
				"receive #{} inputs: {:?}",
				data.len(),
				data.iter().map(|x| x.len()).collect::<Vec<usize>>()
			    );
			    let i: usize = $sid - 1;
			    for (k, v) in data[0].iter().skip(i*6).take(6).enumerate() {
				self.hp_lc[k] = *v;
			    }
			    for (k, v) in data[1].iter().skip(i*27).take(27).enumerate() {
				self.sa_offsetf_cmd[k] = *v;
			    }
			    self
			}
			fn produce(&mut self) -> Option<Vec<Self::O>> {
			    log::debug!("produce");
			    Some(vec![(&self.m1_act_f).into()])
			}
			fn update(&mut self) -> &mut Self {
			    log::debug!("update");
			    self.next();
			    self
			}
		    }
		}
	    )+
	};
    }
    impl_client_for_segments! {1,2,3,4,5,6,7}
}

pub mod assembly {
    use crate::{one_to_many, print_error, Actor, Client};
    pub struct Controller<I, O, const NI: usize, const NO: usize>
    where
        I: Default + std::fmt::Debug,
        O: Default + std::fmt::Debug + Clone,
        Vec<O>: Clone,
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
