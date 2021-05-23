#ifndef __OS_HAL_H__
#define __OS_HAL_H__

#include <stdint.h>

#define os_free free
#define os_malloc malloc

#define WIFI_MGMR_DEFAULT_CLOCK_ID 0
#define WIFI_MGMR_CMD_QUEUE_NAME  "/wlmq"

#define  CODE_WIFI_ON_INIT_DONE   1
#define  CODE_WIFI_ON_MGMR_DONE   2
#define  CODE_WIFI_CMD_RECONNECT  3
#define  CODE_WIFI_ON_CONNECTED   4
#define  CODE_WIFI_ON_DISCONNECT  5
#define  CODE_WIFI_ON_PRE_GOT_IP  6
#define  CODE_WIFI_ON_GOT_IP      7
#define  CODE_WIFI_ON_CONNECTING  8
#define  CODE_WIFI_ON_SCAN_DONE   9
#define  CODE_WIFI_ON_SCAN_DONE_ONJOIN  10
#define  CODE_WIFI_ON_AP_STARTED        11
#define  CODE_WIFI_ON_AP_STOPPED        12
#define  CODE_WIFI_ON_PROV_SSID         13
#define  CODE_WIFI_ON_PROV_BSSID        14
#define  CODE_WIFI_ON_PROV_PASSWD       15
#define  CODE_WIFI_ON_PROV_CONNECT      16
#define  CODE_WIFI_ON_PROV_DISCONNECT   17
#define  CODE_WIFI_ON_PROV_SCAN_START   18
#define  CODE_WIFI_ON_PROV_STATE_GET    19
#define  CODE_WIFI_ON_MGMR_DENOISE      20
#define  CODE_WIFI_ON_AP_STA_ADD        21
#define  CODE_WIFI_ON_AP_STA_DEL        22
#define  CODE_WIFI_ON_EMERGENCY_MAC     23

/* Network Event */
#define EV_NETWORK EV_WIFI
#define CODE_ON_DISCONNECT CODE_WIFI_ON_DISCONNECT
#define CODE_ON_GOT_IP CODE_WIFI_ON_GOT_IP

uint32_t os_get_time_ms(void);

struct os_event {
  int type;
  void *value;
};
typedef struct os_event *os_event_t;

int os_api_init(void);
int os_event_notify(os_event_t);
void os_lock_giant(void);
void os_unlock_giant(void);
int msleep(long msec);

#define os_enter_critical() if (1) {  \
  irqstate_t __irq_state_ = enter_critical_section();

#define os_exit_critical() \
  leave_critical_section(__irq_state_); \
}

#endif
