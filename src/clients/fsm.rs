use crate::{
    impl_read, impl_update, impl_write,
    io::{Data, Read, Write},
    Update,
};
use fsm::{piezostack, positionner, tiptilt};
use std::{ptr, sync::Arc};

/// positionner input
pub enum M2poscmd {}
/// positionner input
pub enum M2posFB {}
/// positionner output
pub enum M2posactF {}
/// piezostack input and tiptilt output
pub enum PZTcmd {}
/// piezostack input
pub enum PZTFB {}
/// piezostack output
pub enum PZTF {}
/// tiptilt input
pub enum TTSP {}
/// tiptilt output
pub enum TTFB {}

impl_update! {positionner}
impl_read! {positionner, (M2poscmd, m2_pos_cmd), (M2posFB, m2_pos_fb)}
impl_write! {positionner, (M2posactF,m2_pos_act_f)}
#[cfg(feature = "fem")]
impl_write! {fem::fem_io::MCM2SmHexF, positionner, (M2posactF,m2_pos_act_f)}
#[cfg(feature = "fem")]
impl_read! {fem::fem_io::MCM2SmHexD,positionner, (M2posFB, m2_pos_fb)}

impl_update! {piezostack}
impl_read! {piezostack, (PZTcmd, pzt_cmd), (PZTFB, pzt_fb)}
impl_write! {piezostack, (PZTF, pzt_f)}
#[cfg(feature = "fem")]
impl_read! {fem::fem_io::MCM2PZTD, piezostack, (PZTFB, pzt_fb)}
#[cfg(feature = "fem")]
impl_write! {fem::fem_io::MCM2PZTF,piezostack, (PZTF, pzt_f)}

impl_update! {tiptilt}
impl_read! {tiptilt, (TTSP, tt_sp), (TTFB, tt_fb)}
impl_write! {tiptilt, (PZTcmd, pzt_cmd)}
