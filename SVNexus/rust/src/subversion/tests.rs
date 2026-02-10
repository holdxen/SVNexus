use super::apr;
use super::context::*;

#[test]
fn uri_is_canonical() {
    let mut pool = apr::PoolFactory::instance().create_pool();
    use super::super::subversion::ffi;

    unsafe {
        let uri = pool
            .string("svn://svn.code.sf.net/p/lame/svn/trunk/lame")
            .unwrap();

        assert!(ffi::svn_uri_is_canonical(uri, pool.as_mut_ptr()) != 0);
    }
}

#[test]
fn create_client() {
    // ContextFactory::instance()
    //     .unwrap()
    //     .create_context(Default::default())
    //     .unwrap();
}

#[test]
fn checkout() {
    tracing_subscriber::fmt().init();
    let url = "https://svn.apache.org/repos/asf/subversion/trunk";

    // let mut opt = CreateContextOptions::default();

    // opt.on_ssl_server_trust_prompt(|_, ssl_failures, _, _| {
    //     Some(TrustServer::new(ssl_failures, true))
    // });
    // opt.on_notify(|notify| {
    //     tracing::info!("on_notify:\n{:#?}", notify);
    // });

    // let mut ctx = ContextFactory::instance()
    //     .unwrap()
    //     .create_context(opt)
    //     .unwrap();

    // let dir = tempfile::tempdir().unwrap();

    // let revision = Revision::Head;

    // let dir = dir.path().to_str().unwrap().to_string();

    // tracing::info!("Checkout: url={}, dir={}", url, dir);

    // let options = CheckoutOptions::new(
    //     url.to_string(),
    //     dir.clone(),
    //     revision,
    //     revision,
    //     Depth::Infinity,
    //     false,
    //     false,
    //     None,
    // );

    // ctx.checkout(options)
    //     .expect(format!("checkout failed to {}", dir).as_str());
}
