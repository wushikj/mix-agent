基于rust的监控探针，主要用于监控硬件及操作系统，监控日志上报至服务端进行分析、计算、告警。
# 探针

## 已完成

* mix_agent_cpu cpu 监控，默认5秒执行一次(收集10次发送)
* mix_agent_machine 获取服务器基本信息，默认1天一次（零点）
* mix_agent_memory 内存监控，默认15秒一次
* mix_agent_disk 磁盘监控，默认30分钟一次
* mix_agent_directory 获取目录信息，默认1天一次（零点）
* mix_agent_process 进程监控，默认10分钟一次
* mix_agent_service windows服务监控，默认10分钟一次

说明：所有探针在安装后会自动执行一次，不需要等到指定的时间。

## 待完成

* mix_agent_updater
* mix_agent_keeper
* mix_agent_network

# 全局配置

以下配置信息保存在 `config/app.yml` 文件


```yml
customer-id: 2acb32811b9c93f6d5d6e90440771398 # 客户编号
project-id: P20200008,P20200009 # 多个使用逗号分隔
mix-endpoint: http://abc.com # 日志接收端点
mix-endpoint-key: 1234513abABqe131413 # 端点密钥
keeper-check-cron: 1/15 * * * * * * # 暂未实现
print-log-json: true # 是否输入日志内容，生产环境建议false
env: dev # 环境，默认dev。prod正式，dev开发、test测试
timeout: 5000 # 接口请求超时配置，毫秒
push-log: true # 是否提交日志，一般只用于开发测试
```

# 探针配置

配置文件均放在config目录下，有些探针强制需要配置文件，有些则不需要。

* global.yml  - 必要，全局配置，主要修改`customer-id`、`project-id`
* mix_agent_machine.yml  - 必要，machine探针使用，主要配置`machine-name`属性，以指示服务器的名称，未指定时web系统中的服务名称将无法显示（为空）
* mix_agent_directory.yml  - directory探针使用，主要配置`root-path`属性，以指示要监控的目录地址
* mix_agent_process.yml  - 进程探针使用，配置要监控的目录进程
* mix_agent_service.yml -  windows服务监控探针使用，配置要监控的目录服务

# 日志格式

```yaml
{
        "batch_id": "20210105200851081379000", //纳秒
        "identity": {
            "customer_id": "460001",
            "project_id": "P12342",
            "target_ip": "192.168.30.11"  //目标IP
        },
        "time": "1610697769239",  //毫秒
        "level": "info",
        "tags": [    //tags的内容可选
            "os|linux",
            "ip|192.168.30.11"
        ],
        "content":"",  //目前只有探针使用此字段
        "category": "",
        "raw_data": {
            "percent_used": 41.39,
            "used": 3.27,
            "free": 4.63,
            "total": 7.91
        },
        "remark": "",  //备注
        "priority": "height",
        "env": "prod",
        "source": {
            "from": "agent",  //日志来源
            "name": "mix_agent_cpu",
            "version": "1.1.2",
            "lang": "java", //开发探针的语言
            "ip": "192.168.30.199" //探针所在主机Ip
        }
    }
```

###  日志来源

可自定义

### level-日志级别

* info
* error
* warn
* trace
* debug

### priority-优先级

* high
* middle
* low


# 部署

## 目录结构

* mix_agent_cpu
* mix_agent_disk
* ....
* mix_agent_keeper
* control.sh
* control.ps1
* shawl.exe windows平台使用的服务安排工具
* config

## 以命令行运行

该方式可用于安装为后台服务前的测试。

### linux

```bash
./mix_agent_xxx
```

### windows

使用 `管理员`权限打开 `powershell`命令行执行相关操作。

```bash
.\mix_agent_xxx
```

说明：以上命令中的xxx代表具体的探针名称。

## 安装为后台服务

### Linux

linux下目前设计了4个控制脚本：

* install.sh  用于安装单个探针
* install_basic_agent.sh  用于安装基础探针（cpu、内存、磁盘、服务器信息）
* query_installed_agent.sh  用于查询本机已安装的探针
* stop_all_agent.sh  用于停止所有探针（即kill -9进程号）

以下仅针对 `install.sh`脚本做说明

```bash
./install.sh agent_name
```

`agent_name`为具体的探针名称

### windows

* control.ps1 用于安装、启用、停止、删除探针服务
* install_basic_agent.ps1  用于安装基础探针（cpu、内存、磁盘、服务器信息）
* query_installed_agent.ps1  用于查询本机已安装的探针
* stop_all_agent.ps1  用于停止所有探针

使用 `管理员`权限打开 `powershell`，转到工作目录下执行   `control.ps1`脚本文件，注意需使用管理员权限。

#### 安装-install

注：安装成功后，会自动启动

```bash
.\control.ps1 -type install -name mix_agent_cpu
```

#### 启动-start

```bash
.\control.ps1 -type start -name mix_agent_cpu
```

#### 启动所有服务

```bash
.\control.ps1 -type start -all
```

#### 重启-restart

```bash
.\control.ps1 -type restart -name mix_agent_cpu
```

#### 重启所有服务

```bash
.\control.ps1 -type restart -all
```

#### 删除-remove

删除时，会自动先停止对应服务再删除

```bash
.\control.ps1 -type remove -name mix_agent_cpu
```

#### 删除所有服务

删除时，会自动先停止对应服务再删除

```bash
.\control.ps1 -type remove -all
```

#### 停止-stop

```bash
.\control.ps1 -type stop -name mix_agent_cpu
```

#### 停止所有服务

```bash
.\control.ps1 -type stop -all
```

#### 查看状态-status

```bash
.\control.ps1 -type status -name mix_agent_cpu
```