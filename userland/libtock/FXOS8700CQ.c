#include <stdio.h>
#include "FXOS8700CQ.h"
#include "math.h"

struct fx0_data {
  int x;
  int y;
  int z;
  bool fired;
};

static struct fx0_data res = { .fired = false };

// internal callback for faking synchronous reads
static void FXOS8700CQ_cb(int x, int y, int z, void* ud) {
  struct fx0_data* result = (struct fx0_data*) ud;
  result->x = x;
  result->y = y;
  result->z = z;
  result->fired = true;
}

double FXOS8700CQ_read_accel_mag() {
  struct fx0_data result = { .fired = false };
  int err;

  err = FXOS8700CQ_subscribe(FXOS8700CQ_cb, (void*)(&result));
  if (err < 0) {
    printf("failed1\n");
    return err;
  }

  printf("ramstart\n");

  err = FXOS8700CQ_start_accel_reading();
    if (err == -10) printf("pend_ram\n");
  else if (err < 0) {
    printf("failed2\n");
    return err;
  }

  yield_for(&result.fired);
  printf("pend_ramdone\n");

  return sqrt(result.x * result.x + result.y * result.y + result.z * result.z);
}

int FXOS8700CQ_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(11, 0, callback, userdata);
}

int FXOS8700CQ_start_accel_reading() {
  return command(11, 0, 0);
}

int FXOS8700CQ_read_acceleration_sync(int* x, int* y, int* z) {
    int err;
    res.fired = false;

    printf("ps\n");

    err = FXOS8700CQ_subscribe(FXOS8700CQ_cb, (void*) &res);
    if (err < 0) {
    printf("failed3\n");
      return err;
    }

    err = FXOS8700CQ_start_accel_reading();
    if (err == -10) printf("pend\n");
    else if (err < 0) {
    printf("failed4\n");

      return err;
    }

    // Wait for the callback.
    yield_for(&res.fired);
    printf("pd\n");

    *x = res.x;
    *y = res.y;
    *z = res.z;

    return 0;
}
