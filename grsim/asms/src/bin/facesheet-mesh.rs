use gmt_fem::FEM;
use matio_rs::{MatFile, MatIO};
use plotters::{prelude::*, style::RED};
use triangle_rs as mesh;

#[derive(Default, Debug, MatIO)]
struct Nodes {
    x: Vec<f64>,
    y: Vec<f64>,
}

fn main() -> anyhow::Result<()> {
    let fem = FEM::from_env()?;
    // println!("{fem}");

    let i = 7;
    println!("Loading nodes from M2Segment{i}AxialD");
    let nodes: Vec<f64> = fem.outputs[9 + i]
        .as_ref()
        .map(|i| i.get_by(|i| i.properties.location.clone()))
        .map(|nodes| {
            nodes
                .into_iter()
                .flat_map(|node| node[..2].to_vec())
                .collect()
        })
        .unwrap();

    let mut builder = mesh::Builder::new();
    builder.add_nodes(&nodes);
    let rim_diameter = 1.0425;
    let delta_rim = 0.01;
    let n_rim = (std::f64::consts::PI * rim_diameter / delta_rim).round();
    let outer_rim: Vec<_> = (0..n_rim as usize)
        .flat_map(|i| {
            let o = 2. * std::f64::consts::PI * i as f64 / n_rim;
            let (s, c) = o.sin_cos();
            let radius = 0.5 * rim_diameter;
            vec![radius * c, radius * s]
        })
        .collect();
    builder.add_polygon(&outer_rim);
    // let switches = format!("pDqa{}", 0.075 );
    let delaunay = builder
        .set_switches(&format!("pDqa{}", 0.0006 / 4f64))
        .build();
    println!("{}", delaunay);
    let mat_nodes = delaunay.vertex_iter().fold(Nodes::default(), |mut n, v| {
        n.x.push(v[0]);
        n.y.push(v[1]);
        n
    });
    MatFile::save(format!("asm{i}_nodes.mat"))?.var("nodes", &mat_nodes)?;

    let fig = SVGBackend::new("asm_facesheet.svg", (768, 768)).into_drawing_area();
    fig.fill(&WHITE).unwrap();

    let xyrange = -0.6..0.6;

    let mut chart = ChartBuilder::on(&fig)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .margin(20)
        .build_cartesian_2d(xyrange.clone(), xyrange)
        .unwrap();
    let mut mesh = chart.configure_mesh();
    mesh.draw().unwrap();

    delaunay
        .triangle_iter()
        .map(|t| {
            t.iter()
                .map(|&i| (delaunay.x()[i], delaunay.y()[i]))
                .collect::<Vec<(f64, f64)>>()
        })
        .into_iter()
        .for_each(|v| {
            chart
                .draw_series(LineSeries::new(
                    v.iter().cycle().take(4).map(|(x, y)| (*x, *y)),
                    &BLACK,
                ))
                .unwrap();
        });

    // let mut colors = colorous::TABLEAU10.iter().cycle();
    // let this_color = colors.next().unwrap().as_tuple();

    chart
        .draw_series(nodes.chunks(2).map(|xy| (xy[0], xy[1])).map(|point| {
            Circle::new(
                point,
                2,
                RED.filled(), // RGBColor(this_color.0, this_color.1, this_color.2).filled(),
            )
        }))
        .unwrap();

    Ok(())
}
