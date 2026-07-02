use image::*;
mod processos;

fn main() {
    let baboon = ImageReader::open("test/baboon.png")
        .expect("unable to open image")
        .decode()
        .expect("unable to decode");
    let _boat = ImageReader::open("test/boat.png")
        .expect("unable to open image")
        .decode()
        .expect("unable to decode");
    processos::salva_decomposicao(
        processos::decomposicao_rgb(baboon),
        "rgb".to_owned(),
        "baboon".to_owned(),
    )
    .unwrap();
}
