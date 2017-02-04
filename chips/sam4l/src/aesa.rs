// adc.rs -- Implementation of SAM4L ADCIFE.
//
// This is a bare-bones implementation of the SAM4L ADC. It is bare-bones
// because it provides little flexibility on how samples are taken. Currently,
// all samples
//   - are 12 bits
//   - use the ground pad as the negative reference
//   - use a VCC/2 positive reference
//   - are right justified
//
// NOTE: The pin labels/assignments on the Firestorm schematic are
// incorrect. The mappings should be
//   AD5 -> ADCIFE channel 6
//   AD4 -> ADCIFE channel 5
//   AD3 -> ADCIFE channel 4
//   AD2 -> ADCIFE channel 3
//   AD1 -> ADCIFE channel 2
//   AD0 -> ADCIFE channel 1
//
// but in reality they are
//   AD5 -> ADCIFE channel 1
//   AD4 -> ADCIFE channel 2
//   AD3 -> ADCIFE channel 3
//   AD2 -> ADCIFE channel 4
//   AD1 -> ADCIFE channel 5
//   AD0 -> ADCIFE channel 6
//
//
//
// Author: Philip Levis <pal@cs.stanford.edu>
// Date: August 5, 2015
//

use core::cell::Cell;
use core::mem;
use kernel::common::volatile_cell::VolatileCell;
use kernel::hil;
use kernel::hil::adc;
use kernel::returncode::ReturnCode;
use nvic;
use pm::{self, Clock, PBAClock};
use scif;

#[repr(C, packed)]
pub struct AesaRegisters {
    // From page 1005 of SAM4L manual
    ctrl: VolatileCell<u32>, // Control               (0x00)
    mode: VolatileCell<u32>, // Mode        (0x04)
    databufptr: VolatileCell<u32>, // Data Buffer Pointer Register                (0x08)
    sr: VolatileCell<u32>, // Status Register          (0x0c)
    ier: VolatileCell<u32>, // interrupt Enable Register  (0x10)
    idr: VolatileCell<u32>, // Interrupt Disable Register  (0x14)
    imr: VolatileCell<u32>, // Interrupt Mask Register          (0x18)
    key0: VolatileCell<u32>, // Key Register 0        (0x20)
    key1: VolatileCell<u32>, // Key Register 1    (0x24)
    key2: VolatileCell<u32>, // Key Register 2       (0x28)
    key3: VolatileCell<u32>, // Key Register 3     (0x2C)
    key4: VolatileCell<u32>, // Key Register 4 (0x30)
    key5: VolatileCell<u32>, // Key Register 5    (0x34)
    key6: VolatileCell<u32>, // Key Register 6    (0x38)
    key7: VolatileCell<u32>, // Key Register 7       (0x3c)
    initvect0: VolatileCell<u32>, // Initialization Vector Register 0        (0x40)
    initvect1: VolatileCell<u32>, // Initialization Vector Register 1          (0x44)
    initvect2: VolatileCell<u32>, // Initialization Vector Register 2      (0x48)
    initvect3: VolatileCell<u32>, // Initialization Vector Register 3      (0x4c) 76

    // this rest are a bit weird
    idata: [VolatileCell<u32>; 4], // Input Data Register      (0x50) 80 this is 16
    odata: [VolatileCell<u32>; 4],// Output Data Register     (0x60) 96  this is 16
    drngseed: [VolatileCell<u32>; 34], // DRNG Seed Register      (0x70) 112 this is 136
    parameter: VolatileCell<u32>, // Parameter Register      (0xf8) 248
    version: VolatileCell<u32>, // Version Register      (0xfc) 252
}

// Page 59 of SAM4L data sheet
const BASE_ADDRESS: *mut AesaRegisters = 0x400B0000 as *mut AesaRegisters;


pub struct Aes_config { //aes_config 
    encrypt_mode: u32, // 0 to decrypt, 1 to encrypt
    key_size: u32, //0 = 128bits, 1 = 192bits, 2 = 256bits
    dma_mode: u32, //0=Non-DMA mode, 1=DMA mode
    opmode: u32, //0 = ECB, 1 = CBC, 2 = OFB, 3 = CFB, 4 = CTR
    cfb_size: u32, //0 = 128bits, 1 = 64bits, 2 = 32bits, 3 = 16bits, 4 = 8bits 
    countermeasure_mask: u32, // [0,15], bit=0 means CounterMeasure is disabled.
}

pub struct Aes_dev_inst { //aes_dev_inst
    registers: *mut AesaRegisters, //Aesa *hw_dev;
    
    encrypt_mode: Cell<u32>, // 0 to decrypt, 1 to encrypt
    key_size: Cell<u32>, //0 = 128bits, 1 = 192bits, 2 = 256bits
    dma_mode: Cell<u32>, //0=Non-DMA mode, 1=DMA mode
    opmode: Cell<u32>, //0 = ECB, 1 = CBC, 2 = OFB, 3 = CFB, 4 = CTR
    cfb_size: Cell<u32>, //0 = 128bits, 1 = 64bits, 2 = 32bits, 3 = 16bits, 4 = 8bits 
    countermeasure_mask: Cell<u32>, // [0,15], bit=0 means CounterMeasure is disabled.
    //aes_config: Cell<Aes_config>, //struct aes_config  *aes_cfg;
}



pub static mut AES_dev_inst: Aes_dev_inst = Aes_dev_inst::new(BASE_ADDRESS);

impl Aes_dev_inst {
    const fn new(base_address: *mut AesaRegisters) -> Aes_dev_inst {
        Aes_dev_inst {
            registers: base_address,
            
            encrypt_mode: Cell::new(1), // 0 to decrypt, 1 to encrypt
            key_size: Cell::new(0), //0 = 128bits, 1 = 192bits, 2 = 256bits
            dma_mode: Cell::new(0), //0=Non-DMA mode, 1=DMA mode
            opmode: Cell::new(0), //0 = ECB, 1 = CBC, 2 = OFB, 3 = CFB, 4 = CTR
            cfb_size: Cell::new(0), //0 = 128bits, 1 = 64bits, 2 = 32bits, 3 = 16bits, 4 = 8bits 
            countermeasure_mask: Cell::new(0x0F), // [0,15], bit=0 means CounterMeasure is disabled
            
            //aes_config: Cell::new(Aes_config{encrypt_mode:0,key_size:0, dma_mode:0,opmode:0,cfb_size:0,  countermeasure_mask: 0  }),
        }
    }
    
    /*
    aes_config.encrypt_mode = 1;
    aes_config.key_size = 0;
    aes_config.dma_mode = 0;
    aes_config.opmode = 0;
    aes_config.cfb_size = 0;
    aes_config.countermeasure_mask = 0x0F; //what does this mean
    */
    
    pub fn aes_get_config_defaults (&self){
        self.encrypt_mode.set(1);
        self.key_size.set(0);
        self.dma_mode.set(0);
        self.opmode.set(0);
        self.cfb_size.set(0);
        self.countermeasure_mask.set(0x0F);
    
    
    }
    
    pub fn aes_set_config(&self){
        let mut value:u32=self.encrypt_mode.get();
        if self.dma_mode.get() != 0 {
            value=value|8;
        }
        
        //check
        value=value| (((0x7 << 4) & ((self.opmode.get()) << 4)));
        value=value| (((0x7 << 8) & ((self.cfb_size.get()) << 8)));
        value=value| (((0xF << 16) & ((self.countermeasure_mask.get()) << 16)));
        unsafe { (*self.registers).mode.set(value) };

    }

}




















