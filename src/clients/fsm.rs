use crate::{
    impl_read, impl_update, impl_write,
    io::{Data, Read, Write},
    Update,
};
use fsm::{piezostack, positionner, tiptilt};
use std::{ptr, sync::Arc};

impl_read! {positionner, (M2poscmd, m2_pos_cmd)}
impl_read! {positionner, (M2posFB, m2_pos_fb)}
impl_write! {positionner, (M2posactF,m2_pos_act_f)}

impl_read! {piezostack, (TTcmd, tt_cmd)}
impl_read! {piezostack, (PZTFB, pzt_fb)}
impl_write! {piezostack, (PZTF, pzt_f)}

impl_read! {tiptilt, (TTSP, tt_sp)}
impl_read! {tiptilt, (TTFB, tt_fb)}
impl_write! {tiptilt, (TTcmd, tt_cmd)}
