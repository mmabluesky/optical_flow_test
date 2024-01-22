class CircularBuffer {
    constructor(size) {
      this.buffer = new Array(size);
      this.index = 0;
    }
  
    add(x) {
      this.buffer[this.index] = x;
      this.index = (this.index + 1) % this.buffer.length;
    }
  
    get() {
      return this.buffer;
    }
  }



  export { CircularBuffer }; 