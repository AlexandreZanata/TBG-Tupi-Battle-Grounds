//! Plain-text localized purpose templates; links never carry credentials.
pub fn body(purpose: &str, locale: &str, code: Option<&str>, url: &str) -> String {
    let pt = locale == "pt-BR";
    let title = match (purpose, pt) {
        ("account_verify", false) => "Verify your email address",
        ("account_verify", true) => "Verifique seu endereco de email",
        ("account_recovery", false) => "Recover your account key",
        ("account_recovery", true) => "Recupere a chave da sua conta",
        ("account_change_old", false) => "Approve or cancel your email change",
        ("account_change_old", true) => "Aprove ou cancele a troca de email",
        ("account_change_new", false) => "Verify your new email address",
        ("account_change_new", true) => "Verifique seu novo endereco de email",
        ("account_email_changed", false) => "Your account email address changed",
        ("account_email_changed", true) => "O email da sua conta mudou",
        ("account_key_rotated", false) => "Your old account key and sessions were revoked",
        ("account_key_rotated", true) => "Sua chave antiga e sessoes foram revogadas",
        ("operator_claim", false) => "Claim or recover your administrator key",
        ("operator_claim", true) => "Obtenha ou recupere sua chave de administrador",
        ("operator_login", false) => "Complete administrator sign in",
        ("operator_login", true) => "Conclua a entrada de administrador",
        ("operator_key_rotated", false) => {
            "Your administrator key changed; previous sessions ended"
        }
        ("operator_key_rotated", true) => {
            "Sua chave de administrador mudou; sessoes anteriores encerradas"
        }
        ("operator_login_notice", false) => "A new administrator session started",
        ("operator_login_notice", true) => "Uma nova sessao de administrador foi iniciada",
        (_, false) => "Account security notification",
        (_, true) => "Notificacao de seguranca da conta",
    };
    let ttl = match purpose {
        "account_recovery" => "15 min",
        "operator_login" => "10 min",
        _ => "24 h",
    };
    let instruction = if pt {
        "Digite este codigo no jogo ou no console"
    } else {
        "Enter this code in the game or console"
    };
    let ignore = if pt {
        "Se voce nao solicitou isto, ignore este email."
    } else {
        "If this was not you, ignore this email."
    };
    let proof = code
        .map(|c| format!("\n{instruction}: {c}\n{ttl}\n"))
        .unwrap_or_default();
    format!("Tupi Battle Grounds\n{title}\n{proof}\n{url}\n\n{ignore}\n")
}
