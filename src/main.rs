// ============================================================================
// PROCESSADOR DE IMAGENS - CONCURRÊNCIA COM TOKIO & SLINT STATUS BAR
// ============================================================================

use slint::Model;
use slint::{ComponentHandle, Image, ModelRc, VecModel};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
mod processos;
slint::include_modules!();

// Cores base do tema Catppuccin Mocha para passar ao Slint
const COR_SUCESSO: slint::Color = slint::Color::from_rgb_u8(166, 227, 161); // Verde
const COR_ERRO: slint::Color = slint::Color::from_rgb_u8(243, 139, 168); // Vermelho
const COR_ALERTA: slint::Color = slint::Color::from_rgb_u8(249, 226, 175); // Amarelo

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    let app_weak = app.as_weak();

    // Armazenamento em memória das matrizes brutas processadas
    let imagem_original = Rc::new(RefCell::new(None::<image::DynamicImage>));
    let historico_rust = Rc::new(RefCell::new(Vec::<(image::DynamicImage, String)>::new()));

    let lista_saidas_model = Rc::new(VecModel::<ImagemSaida>::default());
    app.set_lista_saidas(ModelRc::from(lista_saidas_model.clone()));

    // --- Callback: Abrir Arquivo ---
    let app_clone = app_weak.clone();
    let img_orig_clone = imagem_original.clone();
    let hist_rust_clone = historico_rust.clone();
    let saidas_clone = lista_saidas_model.clone();
    app.on_abrir_arquivo(move || {
        let app = app_clone.unwrap();
        let arquivo_selecionado = rfd::FileDialog::new()
            .set_title("Selecione uma imagem para o PDI")
            .add_filter("Imagens (*.png, *.jpg, *.jpeg)", &["png", "jpg", "jpeg"])
            .pick_file();

        if let Some(caminho) = arquivo_selecionado {
            if let Ok(din_img) = image::open(&caminho) {
                *img_orig_clone.borrow_mut() = Some(din_img.clone());
                hist_rust_clone.borrow_mut().clear();
                saidas_clone.clear();

                let slint_img = converter_para_slint(&din_img);
                app.set_img_entrada(slint_img);
                app.set_status_texto("Imagem de entrada carregada com sucesso!".into());
                app.set_status_cor(COR_SUCESSO.into());
            } else {
                app.set_status_texto(
                    "Erro: Não foi possível ler o arquivo de imagem especificado.".into(),
                );
                app.set_status_cor(COR_ERRO.into());
            }
        }
    });

    // --- Callback: Selecionar Processo ---
    let app_clone = app_weak.clone();
    app.on_selecionar_processo(move |_index, nome_processo| {
        let app = app_clone.unwrap();
        app.set_processo_atual(nome_processo);
        app.set_status_texto(
            format!(
                "Pronto para aplicar o algoritmo: {}",
                app.get_processo_atual()
            )
            .into(),
        );
        app.set_status_cor(slint::Color::from_rgb_u8(205, 214, 244).into());
    });

    // --- Callback: Executar Processo ---
    let app_clone = app_weak.clone();
    let img_orig_clone = imagem_original.clone();
    let hist_rust_clone = historico_rust.clone();
    let saidas_clone = lista_saidas_model.clone();
    app.on_executar_processo(move || {
        let app = app_clone.unwrap();

        if let Some(img_entrada_rust) = img_orig_clone.borrow().clone() {
            let filtro_selecionado = app.get_processo_atual();

            let resultados_pdi = aplicar_algoritmo_pdi(&filtro_selecionado, img_entrada_rust);

            if !resultados_pdi.is_empty() {
                let mut hist_mut = hist_rust_clone.borrow_mut();
                saidas_clone.clear(); // Limpa execuções antigas

                for (nova_img_rust, legenda_txt) in resultados_pdi {
                    let slint_img = converter_para_slint(&nova_img_rust);
                    hist_mut.push((nova_img_rust, legenda_txt.clone()));

                    saidas_clone.push(ImagemSaida {
                        img: slint_img,
                        legenda: slint::SharedString::from(legenda_txt),
                    });
                }

                let total_imagens = saidas_clone.row_count();
                app.set_indice_selecionado((total_imagens - 1) as i32);
                app.set_status_texto(
                    format!(
                        "Sucesso: '{}' aplicado com {} saída(s).",
                        filtro_selecionado, total_imagens
                    )
                    .into(),
                );
                app.set_status_cor(COR_SUCESSO.into());
            }
        } else {
            app.set_status_texto("Erro: Carregue uma imagem de entrada antes de executar.".into());
            app.set_status_cor(COR_ERRO.into());
        }
    });

    // --- Callback: Salvar Apenas a Imagem Atual (Tokio Tasks) ---
    let app_clone = app_weak.clone();
    let hist_rust_clone = historico_rust.clone();
    app.on_salvar_atual(move || {
        let app = app_clone.unwrap();
        let indice = app.get_indice_selecionado() as usize;
        let historico = hist_rust_clone.borrow();

        if let Some((imagem_para_salvar, legenda)) = historico.get(indice) {
            let nome_saneado = legenda
                .to_lowercase()
                .replace(" ", "_")
                .replace("(", "")
                .replace(")", "")
                + ".png";

            let caminho_salvar = rfd::FileDialog::new()
                .set_title("Salvar Imagem Selecionada")
                .add_filter("Imagem PNG (*.png)", &["png"])
                .add_filter("Imagem JPG (*.jpg)", &["jpg"])
                .set_file_name(&nome_saneado)
                .save_file();

            if let Some(caminho) = caminho_salvar {
                let img_clone = imagem_para_salvar.clone();
                let ui_handle = app_clone.clone();

                app.set_status_texto("Salvando arquivo no disco...".into());
                app.set_status_cor(COR_ALERTA.into());

                // Aloca o salvamento pesado no gerenciador de tarefas blocantes do Tokio
                tokio::spawn(async move {
                    let resultado =
                        tokio::task::spawn_blocking(move || img_clone.save(&caminho)).await;

                    // Devolve o feedback para a Thread Principal de UI de forma segura
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            match resultado {
                                Ok(Ok(())) => {
                                    ui.set_status_texto(
                                        "Imagem individual salva com absoluto sucesso!".into(),
                                    );
                                    ui.set_status_cor(COR_SUCESSO.into());
                                }
                                _ => {
                                    ui.set_status_texto(
                                        "Erro crítico ao tentar gravar o arquivo de imagem.".into(),
                                    );
                                    ui.set_status_cor(COR_ERRO.into());
                                }
                            }
                        }
                    })
                    .unwrap();
                });
            }
        }
    });

    // --- Callback: Salvar Todas em ZIP (Tokio Tasks - Sem Corrupção de Cabeçalho) ---
    let app_clone = app_weak.clone();
    let hist_rust_clone = historico_rust.clone();
    app.on_salvar_todas(move || {
        let app = app_clone.unwrap();
        let historico = hist_rust_clone.borrow();
        if historico.is_empty() {
            app.set_status_texto(
                "Aviso: Histórico de processamento vazio. Nada a ser exportado.".into(),
            );
            app.set_status_cor(COR_ALERTA.into());
            return;
        }

        let historico_snapshot: Vec<(image::DynamicImage, String)> = historico.clone();

        let caminho_salvar = rfd::FileDialog::new()
            .set_title("Salvar Todos os Resultados")
            .add_filter("Arquivo ZIP (*.zip)", &["zip"])
            .set_file_name("resultados.zip")
            .save_file();

        if let Some(caminho) = caminho_salvar {
            let ui_handle = app_clone.clone();
            app.set_status_texto(
                format!(
                    "Compactando {} imagens em background...",
                    historico_snapshot.len()
                )
                .into(),
            );
            app.set_status_cor(COR_ALERTA.into());

            tokio::spawn(async move {
                // Executa a compressão I/O pesada isolada de forma assíncrona
                let resultado_zip = tokio::task::spawn_blocking(move || -> Result<(), String> {
                    let arquivo_zip = File::create(&caminho)
                        .map_err(|e| format!("Falha ao criar arquivo: {:?}", e))?;
                    let mut zip = zip::ZipWriter::new(arquivo_zip);
                    let options = zip::write::FileOptions::<()>::default()
                        .compression_method(zip::CompressionMethod::Deflated);

                    for (i, (img_rust, legenda)) in historico_snapshot.iter().enumerate() {
                        let nome_saneado = legenda
                            .to_lowercase()
                            .replace(" ", "_")
                            .replace("(", "")
                            .replace(")", "");
                        let nome_no_zip = format!("{:02}_{}.png", i + 1, nome_saneado);

                        zip.start_file(nome_no_zip, options)
                            .map_err(|e| format!("Erro na estrutura do zip: {:?}", e))?;

                        let mut buffer = std::io::Cursor::new(Vec::new());
                        img_rust
                            .write_to(&mut buffer, image::ImageFormat::Png)
                            .map_err(|e| format!("Erro na codificação PNG: {:?}", e))?;

                        zip.write_all(buffer.get_ref())
                            .map_err(|e| format!("Erro de escrita interna: {:?}", e))?;
                    }

                    // 🔥 Essencial: O zip agora SÓ finaliza de verdade se passar por todo o fluxo sem dar early return
                    zip.finish()
                        .map_err(|e| format!("Erro ao fechar diretório central do ZIP: {:?}", e))?;
                    Ok(())
                })
                .await;

                // Despacha o veredito final para os elementos gráficos da UI
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        match resultado_zip {
                            Ok(Ok(())) => {
                                ui.set_status_texto(
                                    "Arquivo compactado ZIP gerado e salvo com sucesso!".into(),
                                );
                                ui.set_status_cor(COR_SUCESSO.into());
                            }
                            Ok(Err(erro_msg)) => {
                                ui.set_status_texto(
                                    format!("Erro na compactação: {}", erro_msg).into(),
                                );
                                ui.set_status_cor(COR_ERRO.into());
                            }
                            Err(_) => {
                                ui.set_status_texto(
                                    "Erro crítico: A tarefa em background entrou em colapso."
                                        .into(),
                                );
                                ui.set_status_cor(COR_ERRO.into());
                            }
                        }
                    }
                })
                .unwrap();
            });
        }
    });

    app.window().set_fullscreen(true);
    app.run()
}

fn converter_para_slint(din_img: &image::DynamicImage) -> Image {
    let rgba = din_img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();

    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);

    for (src, dest) in rgba.pixels().zip(buffer.make_mut_slice().iter_mut()) {
        *dest = slint::Rgba8Pixel {
            r: src[0],
            g: src[1],
            b: src[2],
            a: src[3],
        };
    }

    Image::from_rgba8(buffer)
}

fn aplicar_algoritmo_pdi(
    filtro: &str,
    img: image::DynamicImage,
) -> Vec<(image::DynamicImage, String)> {
    match filtro {
        "Decomposição RGB" => processos::decomposicao_rgb(img),
        "Decomposição HSV" => processos::decomposicao_hsv(img),
        "Limiarização" => vec![(img, "Resultado Binarizado (Limiar)".to_string())],
        "Transf. Logarítmica" => vec![(img, "Expansão de Tons Escuros (Log)".to_string())],
        "Transf. de Potência (Gamma)" => vec![(img, "Correção Gamma".to_string())],
        "Equalização de Histograma" => vec![(img, "Histograma Uniformizado".to_string())],
        "Fatiamento por Intensidade" => vec![(img, "Destaque de Faixa de Níveis".to_string())],
        "Filtro de Média Gaussiana" => vec![(img, "Suavização Gaussiana".to_string())],
        "Filtro de Mediana" => vec![(img, "Filtro de Mediana (Redução de Ruído)".to_string())],
        "Filtro de Mínimo" => vec![(img, "Filtro de Mínimo (Erosão)".to_string())],
        "Filtro de Máximo" => vec![(img, "Filtro de Máximo (Dilatação)".to_string())],
        "Máscara de Aguçamento" => vec![(img, "Unsharp Masking (High-Boost)".to_string())],
        "Realce por Laplaciano" => vec![(img, "Bordas Laplacianas Somadas".to_string())],
        "Gradiente de Sobel" => vec![(img, "Magnitude do Gradiente de Sobel".to_string())],
        "Passa-Baixa Gaussiano" => vec![(img, "Frequências Altas Atenuadas (Suave)".to_string())],
        "Passa-Alta Gaussiano" => vec![(img, "Frequências Baixas Atenuadas (Bordas)".to_string())],
        "Passa-Baixa Butterworth" => vec![(img, "Filtro Butterworth Passa-Baixa".to_string())],
        "Passa-Alta Butterworth" => vec![(img, "Filtro Butterworth Passa-Alta".to_string())],
        "Filtro Adaptativo de Mediana" => vec![(img, "Filtro de Mediana Adaptativo".to_string())],
        "Ruído Aditivo Gaussiano" => {
            vec![(img, "Imagem Contaminada com Ruído Gaussiano".to_string())]
        }
        "Ruído Sal" => vec![(img, "Ruído Impulsivo Branco (Sal)".to_string())],
        "Ruído Pimenta" => vec![(img, "Ruído Impulsivo Preto (Pimenta)".to_string())],
        "Ruído Sal e Pimenta" => vec![(img, "Ruído Bipolar (Sal e Pimenta)".to_string())],
        _ => {
            println!("Algoritmo não implementado ou desconhecido: {}", filtro);
            vec![(img, "Saída Sem Processamento".to_string())]
        }
    }
}
