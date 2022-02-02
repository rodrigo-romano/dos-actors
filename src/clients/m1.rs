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
