pub mod mount_drives {
    use crate::Client;
    use mount_ctrl::drives;
    impl<'a> Client for drives::Controller<'a> {
        type I = Vec<f64>;
        type O = Vec<f64>;
        fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
            log::debug!(
                "receive #{} inputs: {:?}",
                data.len(),
                data.iter().map(|x| x.len()).collect::<Vec<usize>>()
            );
            for (k, v) in data[0].iter().enumerate() {
                self.cmd[k] = *v;
            }
            for (k, v) in data[1].iter().enumerate() {
                self.oss_az_drive_d[k] = *v;
            }
            for (k, v) in data[2].iter().enumerate() {
                self.oss_el_drive_d[k] = *v;
            }
            for (k, v) in data[3].iter().enumerate() {
                self.oss_gir_drive_d[k] = *v;
            }
            self
        }
        fn produce(&mut self) -> Option<Vec<Self::O>> {
            log::debug!("produce");
            Some(vec![
                (&self.oss_az_drive_f).into(),
                (&self.oss_el_drive_f).into(),
                (&self.oss_gir_drive_f).into(),
            ])
        }
        fn update(&mut self) -> &mut Self {
            log::debug!("update");
            self.next();
            self
        }
    }
}
pub mod mount_ctrlr {
    use crate::Client;
    use mount_ctrl::controller;
    impl<'a> Client for controller::Controller<'a> {
        type I = Vec<f64>;
        type O = Vec<f64>;
        fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
            log::debug!(
                "receive #{} inputs: {:?}",
                data.len(),
                data.iter().map(|x| x.len()).collect::<Vec<usize>>()
            );
            for (k, v) in data[0].iter().enumerate() {
                self.oss_az_drive[k] = *v;
            }
            for (k, v) in data[1].iter().enumerate() {
                self.oss_el_drive[k] = *v;
            }
            for (k, v) in data[2].iter().enumerate() {
                self.oss_gir_drive[k] = *v;
            }
            self
        }
        fn produce(&mut self) -> Option<Vec<Self::O>> {
            log::debug!("produce");
            Some(vec![(&self.cmd).into()])
        }
        fn update(&mut self) -> &mut Self {
            log::debug!("update");
            self.next();
            self
        }
    }
}
