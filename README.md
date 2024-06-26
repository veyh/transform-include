# transform-include

This is a tool for simplifying include paths in C/C++ code. I don't like the fact that some libraries require ALL of their directories to be added as include paths for the library to work just so the author can write `#include "foo.h"` instead of `#include "full/path/to/foo.h"`, which in my opinion is much better because it makes it immediately clear where the file actually is and requires less fiddling with compiler flags.

## Install (x86_64)

```sh
curl -fsSL https://cdn.soupbawx.com/transform-include/transform-include-latest-x86_64-unknown-linux-musl \
  | sudo tee /usr/local/bin/transform-include >/dev/null
  
sudo chmod +x /usr/local/bin/transform-include
```

## Usage

```
Usage: transform-include [OPTIONS] --map <MAP> <FILE>...

Arguments:
  <FILE>...  File(s) to process

Options:
  -n, --dry-run            Don't write to disk, just print the diffs that would happen
  -I, --include <INCLUDE>  Include path used when compiling. Can be specified multiple times
  -m, --map <MAP>          Paths to map to other paths. Format: "/path/to/old:/path/to/new"
  -k, --keep-going         Ignore unresolved include paths instead of exiting
  -h, --help               Print help
```

## Example

```sh
cd /path/to/project

fd -e c -e h -x transform-include \
  -I src \
  -I src/foo \
  -I src/bar \
  --map src:src
```

This would turn

```c
#include "foo.h" // --> src/foo/foo.h
#include "bar.h" // --> src/bar/bar.h
#include "baz.h" // --> src/baz.h
```

into

```c
#include "src/foo/foo.h"
#include "src/bar/bar.h"
#include "src/baz.h"
```

Although I personally prefer to do the following

```sh
cd /path/to/project
mkdir -p deps

cd deps
ln -s .. project

cd ..
fd -e c -e h -x transform-include \
  -I src \
  -I src/foo \
  -I src/bar \
  --map src:project/src
```

which results in

```c
#include "project/src/foo/foo.h"
#include "project/src/bar/bar.h"
#include "project/src/baz.h"
```

and then I just add `/path/to/project/includes` into the include path. Of course, this means that the build pipeline needs to be smart enough not to crash on a cyclic symlink...

## Motivation: Exhibit A, the Nordic nRF5 SDK

For instance, building something with the Nordic nRF5 SDK requires stuff like this

```makefile
CFLAGS += -I$(SDK_ROOT)/components
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_advertising
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_dtm
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_racp
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_ancs_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_ans_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_bas
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_bas_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_cscs
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_cts_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_dfu
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_dis
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_gls
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_hids
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_hrs
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_hrs_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_hts
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_ias
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_ias_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_lbs
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_lbs_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_lls
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_nus
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_nus_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_rscs
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_rscs_c
CFLAGS += -I$(SDK_ROOT)/components/ble/ble_services/ble_tps
CFLAGS += -I$(SDK_ROOT)/components/ble/common
CFLAGS += -I$(SDK_ROOT)/components/ble/nrf_ble_gatt
CFLAGS += -I$(SDK_ROOT)/components/ble/nrf_ble_qwr
CFLAGS += -I$(SDK_ROOT)/components/ble/peer_manager
CFLAGS += -I$(SDK_ROOT)/components/boards
CFLAGS += -I$(SDK_ROOT)/components/iot/common
CFLAGS += -I$(SDK_ROOT)/components/iot/socket/api
CFLAGS += -I$(SDK_ROOT)/components/iot/socket/common
CFLAGS += -I$(SDK_ROOT)/components/libraries/atomic
CFLAGS += -I$(SDK_ROOT)/components/libraries/atomic_fifo
CFLAGS += -I$(SDK_ROOT)/components/libraries/atomic_flags
CFLAGS += -I$(SDK_ROOT)/components/libraries/balloc
CFLAGS += -I$(SDK_ROOT)/components/libraries/bootloader
CFLAGS += -I$(SDK_ROOT)/components/libraries/bootloader/ble_dfu
CFLAGS += -I$(SDK_ROOT)/components/libraries/bootloader/dfu
CFLAGS += -I$(SDK_ROOT)/components/libraries/bsp
CFLAGS += -I$(SDK_ROOT)/components/libraries/button
CFLAGS += -I$(SDK_ROOT)/components/libraries/crc16
CFLAGS += -I$(SDK_ROOT)/components/libraries/crc32
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/cc310
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/cc310_bl
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/cifra
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/mbedtls
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/micro_ecc
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/nrf_hw
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/nrf_sw
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/oberon
CFLAGS += -I$(SDK_ROOT)/components/libraries/crypto/backend/optiga
CFLAGS += -I$(SDK_ROOT)/components/libraries/csense
CFLAGS += -I$(SDK_ROOT)/components/libraries/csense_drv
CFLAGS += -I$(SDK_ROOT)/components/libraries/delay
CFLAGS += -I$(SDK_ROOT)/components/libraries/experimental_section_vars
CFLAGS += -I$(SDK_ROOT)/components/libraries/fds
CFLAGS += -I$(SDK_ROOT)/components/libraries/fifo
CFLAGS += -I$(SDK_ROOT)/components/libraries/fstorage
CFLAGS += -I$(SDK_ROOT)/components/libraries/hardfault
CFLAGS += -I$(SDK_ROOT)/components/libraries/hci
CFLAGS += -I$(SDK_ROOT)/components/libraries/led_softblink
CFLAGS += -I$(SDK_ROOT)/components/libraries/log
CFLAGS += -I$(SDK_ROOT)/components/libraries/log/src
CFLAGS += -I$(SDK_ROOT)/components/libraries/low_power_pwm
CFLAGS += -I$(SDK_ROOT)/components/libraries/mem_manager
CFLAGS += -I$(SDK_ROOT)/components/libraries/memobj
CFLAGS += -I$(SDK_ROOT)/components/libraries/mutex
CFLAGS += -I$(SDK_ROOT)/components/libraries/pwm
CFLAGS += -I$(SDK_ROOT)/components/libraries/pwr_mgmt
CFLAGS += -I$(SDK_ROOT)/components/libraries/ringbuf
CFLAGS += -I$(SDK_ROOT)/components/libraries/scheduler
CFLAGS += -I$(SDK_ROOT)/components/libraries/slip
CFLAGS += -I$(SDK_ROOT)/components/libraries/stack_info
CFLAGS += -I$(SDK_ROOT)/components/libraries/strerror
CFLAGS += -I$(SDK_ROOT)/components/libraries/svc
CFLAGS += -I$(SDK_ROOT)/components/libraries/timer
CFLAGS += -I$(SDK_ROOT)/components/libraries/uart
CFLAGS += -I$(SDK_ROOT)/components/libraries/util
CFLAGS += -I$(SDK_ROOT)/components/softdevice/common
CFLAGS += -I$(SDK_ROOT)/components/softdevice/s140/headers
CFLAGS += -I$(SDK_ROOT)/components/softdevice/s140/headers/nrf52
CFLAGS += -I$(SDK_ROOT)/components/toolchain
CFLAGS += -I$(SDK_ROOT)/components/toolchain/cmsis/include
CFLAGS += -I$(SDK_ROOT)/components/toolchain/gcc
CFLAGS += -I$(SDK_ROOT)/external/fprintf
CFLAGS += -I$(SDK_ROOT)/external/micro-ecc/micro-ecc
CFLAGS += -I$(SDK_ROOT)/external/nano-pb
CFLAGS += -I$(SDK_ROOT)/external/nrf_cc310/include
CFLAGS += -I$(SDK_ROOT)/external/nrf_cc310_bl/include
CFLAGS += -I$(SDK_ROOT)/external/segger_rtt
CFLAGS += -I$(SDK_ROOT)/integration/nrfx
CFLAGS += -I$(SDK_ROOT)/integration/nrfx/legacy
CFLAGS += -I$(SDK_ROOT)/modules/nrfx
CFLAGS += -I$(SDK_ROOT)/modules/nrfx/drivers
CFLAGS += -I$(SDK_ROOT)/modules/nrfx/drivers/include
CFLAGS += -I$(SDK_ROOT)/modules/nrfx/hal
CFLAGS += -I$(SDK_ROOT)/modules/nrfx/mdk
```

Wouldn't it be so much simpler if we could just do

```makefile
CFLAGS += -I$(SDK_ROOT)
```

and call it a day?
