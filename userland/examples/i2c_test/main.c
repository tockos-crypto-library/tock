/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <console.h>
#include <timer.h>
#include <i2c_master_slave.h>

uint8_t slave_read_buf[256];
uint8_t slave_write_buf[256];
uint8_t master_read_buf[256];
uint8_t master_write_buf[256];


int main() {
  i2c_master_slave_set_master_read_buffer(master_read_buf, 256);
  i2c_master_slave_set_master_write_buffer(master_write_buf, 256);
  i2c_master_slave_set_slave_read_buffer(slave_read_buf, 256);
  i2c_master_slave_set_slave_write_buffer(slave_write_buf, 256);
  i2c_master_slave_set_slave_address(0x19);

  printf("hello sender\n");

  while(1) {
      printf("sending\n");
      for(int i = 0; i < 12; i++) {
          master_write_buf[i] = i;
      }
      int result = i2c_master_slave_write_sync(0x18, 12);
      printf("I2C Write: %i\n",result);
      delay_ms(1000);
  }
  return 0;
}
