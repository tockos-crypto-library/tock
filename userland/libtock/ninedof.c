#include <stdio.h>

#include "ninedof.h"
#include "math.h"

struct ninedof_data {
  int x;
  int y;
  int z;
  bool fired;
};

static struct ninedof_data res = { .fired = false };

// internal callback for faking synchronous reads
static void ninedof_cb(int x, int y, int z, void* ud) {
  struct ninedof_data* result = (struct ninedof_data*) ud;
  result->x = x;
  result->y = y;
  result->z = z;
  result->fired = true;
}

double ninedof_read_accel_mag() {
  struct ninedof_data result = { .fired = false };
  int err;

  err = ninedof_subscribe(ninedof_cb, (void*)(&result));
  if (err < 0) {
    return err;
  }

  err = ninedof_start_accel_reading();
  if (err < 0) {
    return err;
  }

  yield_for(&result.fired);

  return sqrt(result.x * result.x + result.y * result.y + result.z * result.z);
}

int ninedof_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(11, 0, callback, userdata);
}

int ninedof_start_accel_reading() {
  return command(11, 1, 0);
}
int ninedof_start_magnetometer_reading() {
  return command(11, 100, 0);
}

int ninedof_read_acceleration_sync(int* x, int* y, int* z) {
    int err;
    res.fired = false;

    err = ninedof_subscribe(ninedof_cb, (void*) &res);
    if (err < 0) return err;

    err = ninedof_start_accel_reading();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&res.fired);

    *x = res.x;
    *y = res.y;
    *z = res.z;

    return 0;
}

int ninedof_read_magenetometer_sync(int* x, int* y, int* z) {
    int err;
    res.fired = false;

    err = ninedof_subscribe(ninedof_cb, (void*) &res);
    if (err < 0) return err;

    err = ninedof_start_magnetometer_reading();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&res.fired);

    *x = res.x;
    *y = res.y;
    *z = res.z;

    return 0;
}
