#[derive(Debug, Clone, Copy)]
pub enum UiRequest {
    CreateGem,
    RingRail,
    Prong,
    Pave,
    Settings,
}

pub fn handle(request: UiRequest) {
    match request {
        UiRequest::CreateGem => {
            println!("[OpenMatrix9 UI] CreateGem");
        }

        UiRequest::RingRail => {
            println!("[OpenMatrix9 UI] RingRail");
        }

        UiRequest::Prong => {
            println!("[OpenMatrix9 UI] Prong");
        }

        UiRequest::Pave => {
            println!("[OpenMatrix9 UI] Pave");
        }

        UiRequest::Settings => {
            println!("[OpenMatrix9 UI] Settings");
        }
    }
}
