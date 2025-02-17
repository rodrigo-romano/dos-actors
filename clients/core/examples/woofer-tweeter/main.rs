use gmt_dos_actors::{actorscript, system::Sys};
use gmt_dos_clients::signals::{Signal, Signals};
use tweeter::ResHiFi;
use woofer::{AddLoFi, AddResLoFi};

mod tweeter;
mod woofer;
// use crate::sys::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sampling_frequency_hz = 1_000.;
    let lofi: Signals = Signals::new(1, 4_000).channels(
        Signal::Sinusoid {
            amplitude: 1.,
            sampling_frequency_hz,
            frequency_hz: 1.,
            phase_s: 0.,
        } + Signal::Sinusoid {
            amplitude: 0.25,
            sampling_frequency_hz,
            frequency_hz: 10.,
            phase_s: 0.,
        },
    );

    let woofer = Sys::new(woofer::Woofer::new()).build()?;
    let tweeter = Sys::new(tweeter::Tweeter::new()).build()?;

    actorscript! {
        #[model(state = running)]
        1: lofi[AddLoFi]~ -> {woofer}[AddResLoFi] -> {tweeter}[ResHiFi]~
    }

    Ok(())
}

#[test]
fn test_main() -> anyhow::Result<()> {
    let sampling_frequency_hz = 1_000.;
    let lofi: Signals = Signals::new(1, 4_000).channels(
        Signal::Sinusoid {
            amplitude: 1.,
            sampling_frequency_hz,
            frequency_hz: 1.,
            phase_s: 0.,
        } + Signal::Sinusoid {
            amplitude: 0.25,
            sampling_frequency_hz,
            frequency_hz: 10.,
            phase_s: 0.,
        },
    );

    let mut woofer = Sys::new(woofer::Woofer::new()).build()?;
    let mut tweeter = Sys::new(tweeter::Tweeter::new()).build()?;

    let mut alofi: Initiator<_> = lofi.into();

    let mut logger: Terminator<_> = Logging::new(1).into();

    alofi
        .add_output()
        .build::<AddLoFi>()
        .into_input(&mut woofer)?;
    woofer
        .add_output()
        .build::<AddResLoFi>()
        .into_input(&mut tweeter)?;
    tweeter
        .add_output()
        .build::<ResHiFi>()
        .into_input(&mut logger)?;

    model!(alofi, woofer, tweeter, logger).check()?.flowchart();

    Ok(())
}

/*
<svg width="130pt" height="271pt"
 viewBox="0.00 0.00 130.00 271.20" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
<g id="graph0" class="graph" transform="scale(1 1) rotate(0) translate(4 267.2)">
<title>G</title>
<polygon fill="#3d3d3d" stroke="transparent" points="-4,4 -4,-267.2 126,-267.2 126,4 -4,4"/>
<!-- 764265725185495747 -->
<g id="node1" class="node">
<title>764265725185495747</title>
<path fill="lightgray" stroke="black" d="M103,-263.2C103,-263.2 19,-263.2 19,-263.2 13,-263.2 7,-257.2 7,-251.2 7,-251.2 7,-239.2 7,-239.2 7,-233.2 13,-227.2 19,-227.2 19,-227.2 103,-227.2 103,-227.2 109,-227.2 115,-233.2 115,-239.2 115,-239.2 115,-251.2 115,-251.2 115,-257.2 109,-263.2 103,-263.2"/>
<text text-anchor="middle" x="61" y="-241.5" font-family="Times,serif" font-size="14.00">Signals</text>
</g>
<!-- 11232177092631395957 -->
<g id="node4" class="node">
<title>11232177092631395957</title>
<ellipse fill="#3d3d3d" stroke="lightgray" cx="61" cy="-188.4" rx="1.8" ry="1.8"/>
</g>
<!-- 764265725185495747&#45;&gt;11232177092631395957 -->
<g id="edge1" class="edge">
<title>764265725185495747&#45;&gt;11232177092631395957</title>
<path fill="none" stroke="#1b9e77" d="M61,-227.16C61,-213.09 61,-194.73 61,-190.36"/>
</g>
<!-- 7219006743340403785 -->
<g id="node2" class="node">
<title>7219006743340403785</title>
<path fill="lightgray" stroke="black" d="M103,-149.6C103,-149.6 19,-149.6 19,-149.6 13,-149.6 7,-143.6 7,-137.6 7,-137.6 7,-125.6 7,-125.6 7,-119.6 13,-113.6 19,-113.6 19,-113.6 103,-113.6 103,-113.6 109,-113.6 115,-119.6 115,-125.6 115,-125.6 115,-137.6 115,-137.6 115,-143.6 109,-149.6 103,-149.6"/>
<text text-anchor="middle" x="61" y="-127.9" font-family="Times,serif" font-size="14.00">WOOFER</text>
</g>
<!-- 4257120465144094172 -->
<g id="node5" class="node">
<title>4257120465144094172</title>
<ellipse fill="#3d3d3d" stroke="lightgray" cx="61" cy="-74.8" rx="1.8" ry="1.8"/>
</g>
<!-- 7219006743340403785&#45;&gt;4257120465144094172 -->
<g id="edge2" class="edge">
<title>7219006743340403785&#45;&gt;4257120465144094172</title>
<path fill="none" stroke="#1b9e77" d="M61,-113.56C61,-99.49 61,-81.13 61,-76.76"/>
</g>
<!-- 1512659184026690350 -->
<g id="node3" class="node">
<title>1512659184026690350</title>
<path fill="lightgray" stroke="black" d="M110,-36C110,-36 12,-36 12,-36 6,-36 0,-30 0,-24 0,-24 0,-12 0,-12 0,-6 6,0 12,0 12,0 110,0 110,0 116,0 122,-6 122,-12 122,-12 122,-24 122,-24 122,-30 116,-36 110,-36"/>
<text text-anchor="middle" x="61" y="-14.3" font-family="Times,serif" font-size="14.00">Logging&lt;f64&gt;</text>
</g>
<!-- 11232177092631395957&#45;&gt;7219006743340403785 -->
<g id="edge3" class="edge">
<title>11232177092631395957&#45;&gt;7219006743340403785</title>
<path fill="none" stroke="#1b9e77" d="M61,-186.45C61,-183.09 61,-171.33 61,-159.65"/>
<polygon fill="#1b9e77" stroke="#1b9e77" points="61,-149.62 65.5,-159.62 61,-154.62 61,-159.62 61,-159.62 61,-159.62 61,-154.62 56.5,-159.62 61,-149.62 61,-149.62"/>
<text text-anchor="middle" x="81" y="-165.9" font-family="Times,serif" font-size="9.00" fill="lightgray">AddLoFi</text>
</g>
<!-- 4257120465144094172&#45;&gt;1512659184026690350 -->
<g id="edge4" class="edge">
<title>4257120465144094172&#45;&gt;1512659184026690350</title>
<path fill="none" stroke="#1b9e77" d="M61,-72.85C61,-69.49 61,-57.73 61,-46.05"/>
<polygon fill="#1b9e77" stroke="#1b9e77" points="61,-36.02 65.5,-46.02 61,-41.02 61,-46.02 61,-46.02 61,-46.02 61,-41.02 56.5,-46.02 61,-36.02 61,-36.02"/>
<text text-anchor="middle" x="89" y="-52.3" font-family="Times,serif" font-size="9.00" fill="lightgray">AddResLoFi</text>
</g>
</g>
</svg>
*/
