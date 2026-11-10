Name:       runedesk
Version:    1.2.4
Release:    0
Summary:    RPM package
License:    GPL-3.0
Requires:   gtk3 libxcb libxdo libXfixes alsa-lib libappindicator-gtk3 libvdpau libva pam gstreamer1-plugins-base
Provides:   libdesktop_drop_plugin.so()(64bit), libdesktop_multi_window_plugin.so()(64bit), libfile_selector_linux_plugin.so()(64bit), libflutter_custom_cursor_plugin.so()(64bit), libflutter_linux_gtk.so()(64bit), libscreen_retriever_plugin.so()(64bit), libtray_manager_plugin.so()(64bit), liburl_launcher_linux_plugin.so()(64bit), libwindow_manager_plugin.so()(64bit), libwindow_size_plugin.so()(64bit), libtexture_rgba_renderer_plugin.so()(64bit)

%description
The best open-source remote desktop client software, written in Rust.

%prep
# we have no source, so nothing here

%build
# we have no source, so nothing here

# %global __python %{__python3}

%install

mkdir -p "%{buildroot}/usr/lib/runedesk" && cp -r ${HBB}/flutter/build/linux/x64/release/bundle/* -t "%{buildroot}/usr/lib/runedesk"
mkdir -p "%{buildroot}/usr/bin"
install -Dm 644 $HBB/res/runedesk.service -t "%{buildroot}/usr/share/runedesk/files"
install -Dm 644 $HBB/res/runedesk.desktop -t "%{buildroot}/usr/share/runedesk/files"
install -Dm 644 $HBB/res/runedesk-link.desktop -t "%{buildroot}/usr/share/runedesk/files"
install -Dm 644 $HBB/res/128x128@2x.png "%{buildroot}/usr/share/icons/hicolor/256x256/apps/runedesk.png"
install -Dm 644 $HBB/res/scalable.svg "%{buildroot}/usr/share/icons/hicolor/scalable/apps/runedesk.svg"

%files
/usr/lib/runedesk/*
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
ln -s /usr/lib/runedesk/runedesk /usr/bin/runedesk
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
    rm /usr/bin/runedesk || true
    update-desktop-database
  ;;
  1)
    # for upgrade
  ;;
esac
