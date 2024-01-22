<template>
  <q-layout view="hHh lpR fFf">
    <q-page-container>
      <q-page class="row items-start q-pa-md">
        <div class="col-8">
          <div id="chartContainer">
            <canvas ref="chartCanvas"></canvas>
          </div>
        </div>

        <div class="col-4">
          <q-card flat bordered class="q-mb-md">
            <!-- 串口列表选择框 -->
            <q-select filled label="选择串口" :options="[...serialPorts]" v-model="selectedPort" />
          </q-card>


          <div class="q-mb-md">
            <q-btn label="开始" color="primary" class="startBtn full-width" @click="start" />
          </div>

          <div class="q-mb-md">
            <q-btn label="停止" color="secondary" class="stopBtn full-width" @click="stop" />
          </div>

          <q-toolbar>
            <q-toolbar-title>日志输出</q-toolbar-title>
          </q-toolbar>

          <!-- Scrollable Log Area -->
          <q-card flat bordered class="q-mb-md">
            <q-scroll-area ref="logScrollArea" class="scroll-area">
              <div>
                <div v-for="(log, index) in logs" :key="index" class="q-pa-md" :class="logClass(log.type)">
                  {{ log.message }}
                </div>
              </div>
            </q-scroll-area>
          </q-card>


        </div>
      </q-page>
    </q-page-container>
  </q-layout>
</template>


<script setup>
import { ref, onMounted, onUnmounted, nextTick  } from 'vue';
import { Chart, registerables } from 'chart.js';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';

import {
  QLayout,
  QPageContainer,
  QPage,
  QSelect,
  QBtn,
  QCard,
  QToolbar,
  QToolbarTitle
} from 'quasar';


Chart.register(...registerables);

let chart;
const buffer_size = 100;  // 缓存大小
const chartCanvas = ref(null);
let optical_flow_coord_x = []  //用于存储光流坐标x
let optical_flow_coord_y = []  //用于存储光流坐标y


let front = 0;
let rear = -1;
let count = 0;
// 初始化队列数组
// const optical_flow_coord_x = new Array(buffer_size);
// const optical_flow_coord_y = new Array(buffer_size);
// let optical_flow_coord_x = new CircularBuffer(buffer_size);
// let optical_flow_coord_y = new CircularBuffer(buffer_size);

const serialPorts = ref([]);

const selectedPort = ref(null);

const logs = ref([]);
const logScrollArea = ref(null);



const data = {
  type: 'line',
  labels: Array.from({ length: buffer_size }, (_, i) => i),
  datasets: [
    {
      label: 'X Coordinates',
      backgroundColor: 'rgba(255, 0, 0, 0.5)',
      borderColor: 'red',
      data: optical_flow_coord_x,//用于存储光流坐标x
      pointRadius: 2, // 设置点的大小
      borderWidth: 1, // 设置连线的宽度
    },
    {
      label: 'Y Coordinates',
      backgroundColor: 'rgba(0, 0, 255, 0.5)',
      borderColor: 'blue',
      data: optical_flow_coord_y, //用于存储光流坐标y
      pointRadius: 2, // 设置点的大小
      borderWidth: 1, // 设置连线的宽度
    }
  ]
};

const options = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    x: {
      type: 'linear',
      beginAtZero: true,
      min: 0,
      max: buffer_size,
      ticks: {
        stepSize: 10 // 设置 X 轴间隔
      }
    },
    y: {
      type: 'linear',
      beginAtZero: true,
      min: -800,
      max: 800,
      ticks: {
        stepSize: 100 // 设置 Y 轴间隔
      }
    }
  }
};



const start = async () => {
  // 开始按钮的逻辑
  if (!selectedPort.value) {
    addToLog('请选择串口', 'error');
    return;
  }

  let result = await invoke('set_port_name', { portName: selectedPort.value.value });
  if (result) {

    addToLog(`设置串口成功：${selectedPort.value.value}`, 'success');
    result = await invoke('get_serial_data', {});
  } else {
    addToLog(`设置串口失败：${selectedPort.value.toString()}`, 'error');
  }

};

const stop = async () => {
  // 停止按钮的逻辑
  console.log('停止');
  try {
    await invoke('close_serial', {});
    addToLog('串口已关闭', 'success');
  } catch (error) {
    addToLog(`关闭串口失败：${error}`, 'error');
  }

};

onMounted(async () => {

  chart = new Chart(chartCanvas.value.getContext('2d'), {
    type: 'line',
    data,
    options
  });

  // let list = ['COM1','COM2','COM3','COM4','COM5','COM6','COM7','COM8','COM9','COM10'];
  let list = [];
  do {
    list = await invoke('get_serial_list', {});
    if (list.length > 0) {
      addToLog(`检测到串口设备：${list.join(', ')}`, 'success');
      break; // 如果检测到串口设备，跳出循环
    }
    addToLog('未检测到串口设备，等待1秒后重试', 'warning');
    await new Promise(resolve => setTimeout(resolve, 1000)); // 等待1秒后再次检查
  } while (true);


  serialPorts.value = list.map((port) => ({ label: port, value: port }));

  listen('serial_data', (event) => {
    const data = JSON.parse(event.payload);
    // console.log(data.x, data.y);
    updateChartData(data.x, data.y);
  });


  listen('serial_error', (event) => {
  // event.payload 包含从 Rust 发送的错误消息
    console.error('Serial port error:', event.payload);
    addToLog(`串口错误：${event.payload}`, 'error');
  });



});



function enqueue(x, y) {
  // 如果队列已满，移除最早的数据点
  if (count >= buffer_size) {
    dequeue();
  }

  // 添加新数据点
  rear = (rear + 1) % buffer_size;
  optical_flow_coord_x[rear] = x;
  optical_flow_coord_y[rear] = y;
  count++;
}

function dequeue() {
  // 移除最早的数据点
  front = (front + 1) % buffer_size;
  count--;
}



let index = 0;

function updateChartData(x, y) {

  addToLog(`光流坐标：${x}, ${y} , 缓存：${optical_flow_coord_x.length}`);
  if (optical_flow_coord_x.length >= buffer_size) {
    optical_flow_coord_x.shift();
    optical_flow_coord_y.shift();
  }

  optical_flow_coord_x.push(x);
  optical_flow_coord_y.push(y);

  // 每次添加新的数据点后立即更新图表
  chart.data.datasets[0].data = [...optical_flow_coord_x];
  chart.data.datasets[1].data = [...optical_flow_coord_y];
  chart.update('none');
  requestAnimationFrame(() => chart.render());
}


// function updateChartData(x, y) {

//   // addToLog(`光流坐标：${x}, ${y}`, 'info');
//   const newX = x;
//   const newY = y;
//   addToLog(`光流坐标：${x}, ${y} , 缓存：${optical_flow_coord_x.length}`);
//   if (optical_flow_coord_x.length >= buffer_size) {
//     optical_flow_coord_x.shift();
//     optical_flow_coord_y.shift();
//   }
//   optical_flow_coord_x.push(newX);
//   optical_flow_coord_y.push(newY);
//   chart.data.datasets[0].data = optical_flow_coord_x;
//   chart.data.datasets[1].data = optical_flow_coord_y;
//   chart.update('none');
//   requestAnimationFrame(() => chart.render());

// }


function addToLog(message, type = 'normal') {
  const now = new Date();
  const timeString = now.toLocaleTimeString('en-US', { hour12: false });
  const newEntry = {
    message: `[${timeString}]: ${message}`,
    type: type,
  };
  logs.value.push(newEntry);

  nextTick(() => {
    const scrollArea = logScrollArea.value.getScrollTarget();
    scrollArea.scrollTop = scrollArea.scrollHeight;
  });
}

// 根据日志类型返回相应的类名
function logClass(type) {
  switch (type) {
    case 'success':
      return 'text-green';
    case 'error':
      return 'text-red';
    case 'warning':
      return 'text-orange';
    default:
      return 'text-black';
  }
}





</script>

<style>
#chartContainer {
  width: 100%;
  height: 90vh;
  /* 根据需要添加更多样式 */
}

#leftArea {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.scroll-area {
  height: 70vh; /* or any other height */
  /* 不能超过窗体边界 */
  max-height: 70vh;
  overflow-y: auto;
}


.text-green {
  color: green;
}
.text-red {
  color: red;
}
.text-orange {
  color: orange;
}
.text-black {
  color: black;
}

</style>
