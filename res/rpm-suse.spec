Name:       runedesk
Version:    1.1.9
Release:    0
Summary:    RPM package
License:    GPL-3.0
Requires:   gtk3 libxcb1 xdotool libXfixes3 alsa-utils libXtst6 libayatana-appindicator3-1 libvdpau1 libva2 pam gstreamer-plugins-base gstreamer-plugin-pipewire

%description
The best open-source remote desktop client software, written in Rust.

%prep
# we have no source, so nothing here

%build
# we have no source, so nothing here

%global __python %{__python3}

%install
mkdir -p %{buildroot}/usr/bin/
mkdir -p %{buildroot}/usr/lib/runedesk/
mkdir -p %{buildroot}/usr/share/runedesk/files/
mkdir -p %{buildroot}/usr/share/icons/hicolor/256x256/apps/
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps/
install -m 755 $HBB/target/release/runedesk %{buildroot}/usr/bin/runedesk
install $HBB/libsciter-gtk.so %{buildroot}/usr/lib/runedesk/libsciter-gtk.so
install $HBB/res/runedesk.service %{buildroot}/usr/share/runedesk/files/
install $HBB/res/128x128@2x.png %{buildroot}/usr/share/icons/hicolor/256x256/apps/runedesk.png
install $HBB/res/scalable.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/runedesk.svg
install $HBB/res/runedesk.desktop %{buildroot}/usr/share/runedesk/files/
install $HBB/res/runedesk-link.desktop %{buildroot}/usr/share/runedesk/files/

%files
/usr/bin/runedesk
/usr/lib/runedesk/libsciter-gtk.so
/usr/share/runedesk/files/runedesk.service
/usr/share/icons/hicolor/256x256/apps/runedesk.png
/usr/share/icons/hicolor/scalable/apps/runedesk.svg
/usr/share/runedesk/files/runedesk.desktop
/usr/share/runedesk/files/runedesk-link.desktop

%changelog
# let's skip this for now

# https://www.cnblogs.com/xingmuxin/p/8990255.html
%pre
# can do something for centos7
case "$1" in
  1)
    # for install
  ;;
  2)
    # for upgrade
    systemctl stop runedesk || true
  ;;
esac

%post
cp /usr/share/runedesk/files/runedesk.service /etc/systemd/system/runedesk.service
cp /usr/share/runedesk/files/runedesk.desktop /usr/share/applications/
cp /usr/share/runedesk/files/runedesk-link.desktop /usr/share/applications/
systemctl daemon-reload
systemctl enable runedesk
systemctl start runedesk
update-desktop-database

%preun
case "$1" in
  0)
    # for uninstall
    systemctl stop runedesk || true
    systemctl disable runedesk || true
    rm /etc/systemd/system/runedesk.service || true
  ;;
  1)
    # for upgrade
  ;;
esac

%postun
case "$1" in
  0)
    # for uninstall
    rm /usr/share/applications/runedesk.desktop || true
    rm /usr/share/applications/runedesk-link.desktop || true
    update-desktop-database
  ;;
  1)
    # for upgrade
  ;;
esac
