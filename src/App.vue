<template>
  <div id="container">
    <div id="chartContainer">
      <canvas ref="chartCanvas"></canvas>
    </div>

    <div id="leftArea">
      <h2>当前串口号：{{ portName }}</h2>
      <h2>日志输出</h2>
      <div id="logBox"></div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { Chart, registerables } from 'chart.js';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';

Chart.register(...registerables);

let chart;
const buffer_size = 100;  // 缓存大小
const chartCanvas = ref(null);
let optical_flow_coord_x = []  //用于存储光流坐标x
let optical_flow_coord_y = []  //用于存储光流坐标y

const portName = ref('');


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

onMounted(async () => {
  chart = new Chart(chartCanvas.value.getContext('2d'), {
    type: 'line',
    data,
    options
  });


  const result = await invoke('get_serial_data', {});

  portName.value = await invoke('get_port_name', {});

  listen('serial_data', (event) => {
    const data = JSON.parse(event.payload);
    // console.log(data.x, data.y);
    updateChartData(data.x, data.y);
  });




});



function updateChartData(x, y) {

  const newX = x//Math.random() * 1600 - 800; // 举例
  const newY = y//Math.random() * 1600 - 800; // 举例
  addToLog(newX, newY, optical_flow_coord_x.length);
  // 如果数据已满，则移除最早的数据点
  if (optical_flow_coord_x.length >= buffer_size) {
    optical_flow_coord_x.shift();
    optical_flow_coord_y.shift();
  }

  // 添加新数据点
  optical_flow_coord_x.push(newX);
  optical_flow_coord_y.push(newY);

  // 更新图表数据集
  chart.data.datasets[0].data = optical_flow_coord_x;
  chart.data.datasets[1].data = optical_flow_coord_y;

  // 通知图表更新
  chart.update();
}


function addToLog(x, y, size) {
  const logBox = document.getElementById('logBox');
  const newEntry = document.createElement('div');

  // 获取当前时间并格式化为24小时制本地时间字符串，精确到秒
  const now = new Date();
  const timeString = now.toLocaleTimeString('en-US', { hour12: false });

  newEntry.textContent = `[${timeString}]: X: ${x}, Y: ${y} Size: ${size}`;
  logBox.appendChild(newEntry);

  // 滚动到日志框的底部
  logBox.scrollTop = logBox.scrollHeight;
}


</script>

<style>

* {
  box-sizing: border-box; /* 确保边框和内边距包含在宽度和高度内 */
}

html, body {
  margin: 0;
  padding: 0;
  height: 100vh; /* 视窗高度 */
  overflow: hidden; /* 防止在body级别出现滚动条 */
}

#container {
  display: flex;
  flex-direction: row;
  height: 100vh; /* 视窗高度 */
  overflow: hidden;
}

#chartContainer {
  flex: 2;
  padding: 10px;
  height: 100%; /* 容器高度 */
  overflow: hidden;
}

#leftArea {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 10px;
  height: 100%; /* 容器高度 */
  overflow: hidden;
}

#logBox {
  overflow-y: auto;
  height: calc(100% - 20px); /* 减去标题等元素的高度 */
}


</style>
