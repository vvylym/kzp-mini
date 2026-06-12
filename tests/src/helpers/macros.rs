//! Test boilerplate macros.

/// Declares an async integration test with [`TestApp`](crate::helpers::TestApp) and the module's `universe`.
///
/// Expects a `Universe` struct and `universe` function (or variant) in the same module.
/// Applies `#[tokio::test]` - do not add it yourself.
///
/// Document each test with regular `//` comments (**Spec**, **Given**, **When**, **Then**) immediately
/// above the macro invocation - doc comments (`///`) are not attached to the generated test fn.
///
/// # Examples
///
/// ```ignore
/// // Spec: nominal - member joins the pool.
/// //
/// // Given:
/// // - An initialized pool and funded user
/// //
/// // When:
/// // - User calls `join_pool`
/// //
/// // Then:
/// // - Member PDA is created
/// with_universe!(join_pool_nominal, |app, u| {
///     let (bob, bob_ata) = app.create_funded_user(500_000_000).await;
///     app.join_member(&bob, u.pool, bob_ata).await;
/// });
///
/// with_universe!(deposit_savings_nominal, universe(500_000_000), |app, u| {
///     app.deposit(&u.member, u.pool, u.member_ata, 200_000_000).await;
/// });
/// ```
#[macro_export]
macro_rules! with_universe {
    (
        $name:ident,
        |$app:ident, $u:ident| $body:block
    ) => {
        #[::tokio::test]
        async fn $name() {
            let mut __app = $crate::helpers::TestApp::new().await;
            let $u = universe(&mut __app).await;
            let $app = &mut __app;
            $body
        }
    };
    (
        $name:ident,
        $setup:ident,
        |$app:ident, $u:ident| $body:block
    ) => {
        #[::tokio::test]
        async fn $name() {
            let mut __app = $crate::helpers::TestApp::new().await;
            let $u = $setup(&mut __app).await;
            let $app = &mut __app;
            $body
        }
    };
    (
        $name:ident,
        $setup:ident($($arg:expr),+ $(,)?),
        |$app:ident, $u:ident| $body:block
    ) => {
        #[::tokio::test]
        async fn $name() {
            let mut __app = $crate::helpers::TestApp::new().await;
            let $u = $setup(&mut __app, $($arg),+).await;
            let $app = &mut __app;
            $body
        }
    };
}
