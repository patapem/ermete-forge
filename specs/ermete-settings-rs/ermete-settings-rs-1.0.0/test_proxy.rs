#[zbus::dbus_proxy(interface = "org.freedesktop.Accounts.User")]
trait AccountsUser {}

fn main() {
    let iface = AccountsUserProxy::INTERFACE;
}
