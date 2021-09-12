/****************************************************************************
 * arch/risc-v/src/bl602/bl_os_adapter/bl_os_system.h
 *
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.  The
 * ASF licenses this file to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance with the
 * License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
 * License for the specific language governing permissions and limitations
 * under the License.
 *
 ****************************************************************************/

#ifndef _BL_OS_SYSTEM_H_
#define _BL_OS_SYSTEM_H_

#include <bl_os_adapter/bl_os_adapter.h>

#ifdef __cplusplus
extern "C" {
#endif

/****************************************************************************
 * Definition
 ****************************************************************************/

#undef assert
#define assert(f)                                                         \
    do {                                                                  \
        if (!(f)) {                                                       \
            g_bl_ops_funcs._assert(__FILE__, __LINE__, __FUNCTION__, #f); \
        }                                                                 \
    } while (0)

// #define bl_os_printf g_bl_ops_funcs._printf

/****************************************************************************
 * Private Types
 ****************************************************************************/

#ifdef __cplusplus
}
#endif

#endif /* _BL_OS_SYSTEM_H_ */