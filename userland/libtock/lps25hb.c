#include "tock.h"
#include "lps25hb.h"

struct lps25hb_data {
  bool fired;
  int value;
};

static struct lps25hb_data result = { .fired = false };

// Internal callback for faking synchronous reads
static void lps25hb_cb(int value,
                       __attribute__ ((unused)) int unused1,
                       __attribute__ ((unused)) int unused2,
                       void* ud) {
  struct lps25hb_data* result = (struct lps25hb_data*) ud;
  result->value = value;
  result->fired = true;
}

int lps25hb_set_callback (subscribe_cb callback, void* callback_args) {
    return subscribe(DRIVER_NUM_LPS25HB, 0, callback, callback_args);
}

int lps25hb_get_pressure () {
    return command(DRIVER_NUM_LPS25HB, 1, 0);
}

int lps25hb_get_pressure_sync () {
    int err;
    result.fired = false;

    err = lps25hb_set_callback(lps25hb_cb, (void*) &result);
    if (err < 0) return err;

    err = lps25hb_get_pressure();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&result.fired);

    return result.value;
}
