#
# TrafficMon Rust Package
#

include $(TOPDIR)/rules.mk

PKG_NAME:=trafficmon
PKG_VERSION:=0.1.0
PKG_RELEASE:=1

include $(INCLUDE_DIR)/package.mk

define Package/trafficmon
  SECTION:=utils
  CATEGORY:=Utilities
  TITLE:=Rust-based NFT traffic monitor
  DEPENDS:=+libc
endef

define Package/trafficmon/install
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/target/release/trafficmon $(1)/usr/bin/trafficmon

	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/trafficmon.init $(1)/etc/init.d/trafficmon
endef

define Build/Compile
	cd $(PKG_BUILD_DIR) && cargo build --release
endef

$(eval $(call BuildPackage,trafficmon))
