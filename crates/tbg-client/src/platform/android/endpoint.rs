//! Android transport policy accepts only TLS or an explicit internal loopback contract.

pub fn allowed(raw: &str, internal_loopback: bool, contract: Option<&str>) -> bool {
    let Ok(url) = url::Url::parse(raw) else {
        return false;
    };
    let query_allowed = match url.query() {
        None => true,
        Some(query) => contract.is_some_and(|id| query == format!("contract={id}")),
    };
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !query_allowed
    {
        return false;
    }
    url.scheme() == "wss"
        || (internal_loopback && url.scheme() == "ws" && url.host_str() == Some("127.0.0.1"))
}

#[cfg(test)]
mod tests {
    use super::allowed;

    #[test]
    fn cleartext_is_internal_literal_loopback_only() {
        // P69/AN13: public transports require TLS; no release cleartext exception.
        assert!(allowed("ws://127.0.0.1:18069/ws", true, None));
        assert!(!allowed("ws://127.0.0.1:18069/ws", false, None));
        assert!(!allowed("ws://localhost:18069/ws", true, None));
        assert!(!allowed("ws://example.com/ws", true, None));
        assert!(allowed("wss://example.com/ws", false, None));
    }

    #[test]
    fn admitted_socket_preserves_only_its_exact_contract_query() {
        // P66/AN11: admission adds the shared contract; Android must not block that socket.
        let id = "w30-r2-3d56156049e626c5";
        let socket = format!("wss://example.com/ws?contract={id}");
        assert!(allowed(&socket, false, Some(id)));
        assert!(!allowed(&socket, false, None));
        for query in [
            "contract=other",
            "contract=other&token=secret",
            "token=secret",
        ] {
            assert!(!allowed(
                &format!("wss://example.com/ws?{query}"),
                false,
                Some(id)
            ));
        }
        assert!(!allowed(
            &format!("{socket}&contract={id}"),
            false,
            Some(id)
        ));
    }

    #[test]
    fn transport_rejects_embedded_credentials_and_fragments() {
        for raw in [
            "wss://user:secret@example.com/ws",
            "wss://example.com/ws#secret",
            "https://example.com/ws",
        ] {
            assert!(!allowed(raw, false, None));
        }
    }
}
