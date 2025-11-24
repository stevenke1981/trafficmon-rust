#
# TrafficMon Rust Package
#

include $(TOPDIR)/rules.mk

PKG_NAME:=trafficmon
PKG_VERSION:=1.0.0
PKG_RELEASE:=1

include $(INCLUDE_DIR)/package.mk
include $(TOPDIR)/feeds/packages/lang/rust/rust-package.mk

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

$(eval $(call RustBinPackage,trafficmon))
$(eval $(call BuildPackage,trafficmon))