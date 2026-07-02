use slint::{Image, ModelRc, SharedString, VecModel};
use std::rc::Rc;
mod processos;
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // 1. Injetando a lista de processos
    let processos_vetor = vec![
        SharedString::from("Filtro de Sobel"),
        SharedString::from("Desfoque Gaussiano"),
        SharedString::from("Limiarização (Otsu)"),
        SharedString::from("Espelhamento Horizontal"),
    ];
    let modelo_processos = Rc::new(VecModel::from(processos_vetor));
    ui.set_lista_processos(ModelRc::from(modelo_processos));

    // 2. Evento de abrir arquivo (.png)
    ui.on_abrir_arquivo({
        let ui_handle = ui.as_weak();
        move || {
            let arquivo = rfd::FileDialog::new()
                .add_filter("Imagens PNG", &["png"])
                .set_title("Selecione uma Imagem de Entrada")
                .pick_file();

            if let Some(caminho) = arquivo {
                println!("Arquivo selecionado: {:?}", caminho);
                match Image::load_from_path(&caminho) {
                    Ok(img) => {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_img_entrada(img);
                        }
                    }
                    Err(err) => eprintln!("Erro ao carregar imagem: {:?}", err),
                }
            }
        }
    });

    // 3. Evento de seleção da lista lateral
    ui.on_selecionar_processo({
        let ui_handle = ui.as_weak();
        move |_index, nome| {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_processo_atual(nome);
            }
        }
    });

    // 4. Tratando a execução do processo científico/filtro
    ui.on_executar_processo({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let filtro = ui.get_processo_atual();
                println!(
                    "▶️ Backend Rust iniciando a execução do algoritmo: {}",
                    filtro
                );

                // TODO: Sua lógica pesada de processamento entra aqui.
                // Exemplo:
                // let img_processada = meu_modulo::aplicar_filtro(&ui.get_img_entrada(), &filtro);
                // ui.set_img_saida(img_processada);
            }
        }
    });

    ui.run()
}
