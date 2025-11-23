include $(TOPDIR)/rules.mk

PKG_NAME:=trafficmon
PKG_VERSION:=0.1.0
PKG_RELEASE:=1

PKG_BUILD_DIR := $(BUILD_DIR)/$(PKG_NAME)

include $(INCLUDE_DIR)/package.mk

define Package/trafficmon
  SECTION:=utils
  CATEGORY:=Utilities
  TITLE:=Traffic Monitor (Rust) for OpenWrt
  DEPENDS:=+libc +nftables +conntrack
endef

define Package/trafficmon/description
Rust-based traffic monitor using nftables counters for eth1 and pppoe-wan.
endef

define Build/Prepare
	mkdir -p $(PKG_BUILD_DIR)
	cp -r ../src $(PKG_BUILD_DIR)/
	cp ../Cargo.toml $(PKG_BUILD_DIR)/
endef

define Build/Compile
	cd $(PKG_BUILD_DIR) && cargo build --release
endef

define Package/trafficmon/install
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/target/release/trafficmon $(1)/usr/bin/trafficmon

	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/etc/init.d/trafficmon $(1)/etc/init.d/trafficmon
endef

$(eval $(call BuildPackage,trafficmon))
