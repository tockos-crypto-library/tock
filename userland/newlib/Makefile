NEWLIB_REVISION ?= 52a6da816f72e

# n.b. newlib LTO support requires current at least through 52a6da816f72e
# that pulls in some fixes. Prior to those patches, newlib compiles fine,
# but the final link step would fail for some applications

all: rebuild-newlib

newlib-git:
	@git clone git://sourceware.org/git/newlib-cygwin.git $@
	@pushd $@ && git checkout $(NEWLIB_REVISION)

rebuild-newlib: newlib-git
	@rm -rf newlib-$(NEWLIB_REVISION)-out
	@mkdir -p newlib-$(NEWLIB_REVISION)-out
	@echo "Entering directory newlib-$(NEWLIB_REVISION)-out"
	cd newlib-$(NEWLIB_REVISION)-out; sh ../build.sh ../$<
	@echo "Copying built artifacts"
	@cp newlib-$(NEWLIB_REVISION)-out/arm-none-eabi/thumb/v6-m/newlib/libc.a cortex-m0/
	@cp newlib-$(NEWLIB_REVISION)-out/arm-none-eabi/thumb/v6-m/newlib/libg.a cortex-m0/
	@cp newlib-$(NEWLIB_REVISION)-out/arm-none-eabi/thumb/v6-m/newlib/libm.a cortex-m0/
	@cp newlib-$(NEWLIB_REVISION)-out/arm-none-eabi/thumb/v7e-m/newlib/libc.a cortex-m4/
	@cp newlib-$(NEWLIB_REVISION)-out/arm-none-eabi/thumb/v7e-m/newlib/libg.a cortex-m4/
	@cp newlib-$(NEWLIB_REVISION)-out/arm-none-eabi/thumb/v7e-m/newlib/libm.a cortex-m4/

