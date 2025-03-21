/*!
# GMT DOS Actors Scope

`gmt_dos-clients_scope` is a client/server graphical display implementation for [gmt_dos-actors](https://docs.rs/gmt_dos-actors/) models.

`gmt_dos-clients_scope` has 2 features: `server` and `client`.
The `server` feature need to be enabled only on the server applications:
```shell
cargo add gmt_dos-clients_scope --features=server
```
and the `client` feature only on the machine displaying the scopes:
```shell
cargo add gmt_dos-clients_scope --features=client
```
When both the server and the client are run on the same local machine, the IP address of the server is set to `127.0.0.1`
and the client address is set to `0.0.0.0:0`.
If you want to run the server on a different remote machine,
you need to set the server IP address with the environment variable `SCOPE_SERVER_IP` on both the server and the client.

For a server running in the AWS cloud, on an AWS instance, the server IP address is set
to the private IP address of the instance whereas the server IP address is set
to the public IP address of the instance on the client machine.
*/

pub use gmt_dos_clients_scopehub::scopehub;

const SERVER_IP: &'static str = "127.0.0.1";
#[cfg(feature = "client")]
const CLIENT_ADDRESS: &'static str = "0.0.0.0:0";

mod payload;

/// Marker for scopes that display signals
#[derive(Debug)]
pub enum PlotScope {}
/// Marker for scopes that display an image
#[derive(Debug)]
pub enum ImageScope {}
/// Marker for scopes that display an image with a mask applied to it
#[derive(Debug)]
pub enum GmtScope {}

/// Scopes marker trait
pub trait ScopeKind {
    fn window_size() -> (f32, f32);
}
impl ScopeKind for PlotScope {
    fn window_size() -> (f32, f32) {
        (800f32, 600f32)
    }
}
impl ScopeKind for ImageScope {
    fn window_size() -> (f32, f32) {
        (800f32, 800f32)
    }
}
impl ScopeKind for GmtScope {
    fn window_size() -> (f32, f32) {
        (800f32, 900f32)
    }
}
/// Image scopes marker trait
pub trait ImageScopeKind: ScopeKind {}
impl ImageScopeKind for ImageScope {}
impl ImageScopeKind for GmtScope {}

#[cfg(any(feature = "client", doc))]
pub mod client;

#[cfg(any(feature = "server", doc))]
pub mod server;
