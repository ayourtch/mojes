//! Tests for the transpilation patterns exercised by the mojes-conf WebRTC
//! conference client: the `JSON` global, zero-arg `Element.remove()`, async
//! Promise `.await`, trickle-ICE closures, and roster iteration. Each test
//! both checks the generated JS shape and, where practical, runs it under Boa.

#[cfg(test)]
mod conf_patterns {
    use boa_engine::{Context, Source};
    use mojes_mojo::*;
    use syn::{parse_quote, Block, ItemImpl};

    fn eval(code: &str) -> Result<boa_engine::JsValue, String> {
        let mut ctx = Context::default();
        ctx.eval(Source::from_bytes(code))
            .map_err(|e| format!("{e}"))
    }

    fn block_js(b: &Block) -> String {
        rust_block_to_js(b)
    }

    #[test]
    fn json_stringify_and_parse_are_native() {
        let b: Block = parse_quote!({
            let s = JSON.stringify(&value);
            let parsed: Thing = JSON.parse(&s);
        });
        let js = block_js(&b);
        assert!(
            js.contains("JSON.stringify(value)"),
            "expected native JSON.stringify, got:\n{js}"
        );
        assert!(
            js.contains("JSON.parse(s)"),
            "expected native JSON.parse, got:\n{js}"
        );
        // Round-trips under a real JS engine.
        let prog = "const value = {a: 1, b: 'x'};".to_string()
            + &js.replace(": Thing", "")
            + "\n JSON.stringify(parsed);";
        // strip the Rust type annotation artifact if present
        let out = eval(&prog.replace("let parsed: Thing", "let parsed")).unwrap();
        let mut ctx = Context::default();
        assert_eq!(out.to_string(&mut ctx).unwrap().to_std_string().unwrap(), "{\"a\":1,\"b\":\"x\"}");
    }

    #[test]
    fn element_remove_zero_arg_is_method_call() {
        // Zero-arg remove() (DOM Element.remove) must be a plain method call,
        // NOT the HashMap/array splice-or-delete helper.
        let b: Block = parse_quote!({
            el.remove();
        });
        let js = block_js(&b);
        assert!(js.contains("el.remove()"), "got:\n{js}");
        assert!(!js.contains("splice"), "zero-arg remove must not use splice helper:\n{js}");
    }

    #[test]
    fn map_remove_one_arg_still_uses_helper() {
        // One-arg remove() (HashMap::remove) must keep the universal helper.
        let b: Block = parse_quote!({
            let old = peers.remove(&id);
        });
        let js = block_js(&b);
        assert!(js.contains("splice"), "one-arg remove should use splice/delete helper:\n{js}");
    }

    #[test]
    fn await_lowers_to_native_await_in_async_fn() {
        let b: Block = parse_quote!({
            let offer = pc.createOffer().await;
            pc.setLocalDescription(&offer).await;
            let answer = pc.createAnswer().await;
        });
        let js = block_js(&b);
        assert!(js.contains("await pc.createOffer()"), "got:\n{js}");
        assert!(js.contains("await pc.setLocalDescription(offer)"), "got:\n{js}");
        assert!(js.contains("await pc.createAnswer()"), "got:\n{js}");
    }

    #[test]
    fn description_object_literal_shape() {
        // SessionDesc { type, sdp } builds a positional constructor call.
        let i: ItemImpl = parse_quote! {
            impl Foo {
                fn make(sdp: String) -> SessionDesc {
                    SessionDesc { r#type: "offer".to_string(), sdp }
                }
            }
        };
        let js = generate_js_methods_for_impl(&i);
        assert!(js.contains("new SessionDesc("), "got:\n{js}");
        // r#type must have been unraw-ed to `type`
        assert!(!js.contains("r#type"), "raw identifier leaked:\n{js}");
    }

    #[test]
    fn icecandidate_closure_reads_optional_candidate() {
        // The trickle-ICE handler: match on ev.candidate Option.
        let b: Block = parse_quote!({
            pc.addEventListener("icecandidate", move |ev: RTCPeerConnectionEvent| {
                match ev.candidate {
                    Some(c) => { ws.lock().unwrap().send(&c.candidate); }
                    None => {}
                }
            });
        });
        let js = block_js(&b);
        assert!(js.contains("addEventListener(\"icecandidate\""), "got:\n{js}");
        assert!(js.contains("ev.candidate"), "got:\n{js}");
        // The closure becomes an arrow function passed as the 2nd argument.
        assert!(js.contains("=>"), "expected arrow closure:\n{js}");
    }

    #[test]
    fn roster_iteration_awaits_each_peer() {
        let b: Block = parse_quote!({
            let peers = msg.peers.unwrap_or(vec![]);
            for p in peers {
                ensure_peer(p.id.clone(), true).await;
            }
        });
        let js = block_js(&b);
        // Result-aware unwrap_or: null/undefined and {error: ..} take the
        // default, {ok: v} unwraps, plain arrays pass through.
        assert!(
            js.contains("(msg.peers, [])"),
            "unwrap_or(vec![]) should dispatch with [] as the default:\n{js}"
        );
        assert!(js.contains("for (const p of peers)"), "got:\n{js}");
        assert!(js.contains("await ensure_peer("), "got:\n{js}");
    }

    #[test]
    fn generated_conf_snippet_is_valid_js() {
        // A representative fragment must at least parse/run under Boa when the
        // referenced globals are stubbed.
        let b: Block = parse_quote!({
            let t = msg.r#type.clone();
            if t == "offer" {
                let sdp = msg.sdp.unwrap_or("".to_string());
                let desc = SessionDesc { r#type: "offer".to_string(), sdp };
            }
        });
        let js = block_js(&b);
        let prog = format!(
            "class SessionDesc {{ constructor(type, sdp) {{ this.type = type; this.sdp = sdp; }} }}\n\
             const msg = {{ type: 'offer', sdp: 'x' }};\n{js}\n'ok';"
        );
        assert!(eval(&prog).is_ok(), "generated snippet failed to run:\n{prog}");
    }
}
