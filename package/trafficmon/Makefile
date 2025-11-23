#
# TrafficMon Rust Package
#
include $(TOPDIR)/rules.mk

PKG_NAME:=trafficmon
PKG_VERSION:=0.1.0
PKG_RELEASE:=1

PKG_SOURCE_PROTO:=git
PKG_SOURCE_URL:=file://$(TOPDIR)/../
PKG_SOURCE_VERSION:=HEAD
PKG_MIRROR_HASH:=skip

PKG_BUILD_DEPENDS:=rust/host

include $(INCLUDE_DIR)/package.mk
include $(TOPDIR)/feeds/packages/lang/rust/rust-package.mk

define Package/trafficmon
  SECTION:=utils
  CATEGORY:=Utilities
  TITLE:=Rust-based NFT traffic monitor
  DEPENDS:=+libc
  URL:=https://github.com/yourusername/trafficmon-rust
endef

define Package/trafficmon/description
  Monitor traffic on eth1 and pppoe-wan using nftables counters.
endef

define Package/trafficmon/install
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_INSTALL_DIR)/bin/trafficmon $(1)/usr/bin/trafficmon
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/trafficmon.init $(1)/etc/init.d/trafficmon
endef

$(eval $(call BuildPackage,trafficmon))
