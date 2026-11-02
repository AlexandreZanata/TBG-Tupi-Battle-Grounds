//! Plain-text localized purpose templates; links never carry credentials.

pub fn body(purpose: &str, locale: &str, code: Option<&str>, url: &str) -> String {
    let (heading, instruction, ignore) = if locale == "pt-BR" {
        (
            "Tupi Battle Grounds",
            "Digite este codigo no jogo ou no console",
            "Se voce nao solicitou isto, ignore este email.",
        )
    } else {
        (
            "Tupi Battle Grounds",
            "Enter this code in the game or console",
            "If this was not you, ignore this email.",
        )
    };
    let ttl = match purpose {
        "account_recovery" => "15 min",
        "operator_login" => "10 min",
        _ => "24 h",
    };
    let proof = code
        .map(|c| format!("\n{instruction}: {c}\n{ttl}\n"))
        .unwrap_or_default();
    format!("{heading}\n{purpose}\n{proof}\n{url}\n\n{ignore}\n")
}
