import { apiClient } from './client'
import type {
  RegisterDeviceResponse,
  ConnectionTestResult,
  PairResponse,
  PairHostRequest,
  DaemonHelloRequest,
} from './types'

export const authApi = {
  // Test server connection
  async testConnection(serverUrl: string): Promise<ConnectionTestResult> {
    const client = apiClient
    client.setBaseUrl(serverUrl)

    try {
      // Try to fetch server settings/health endpoint
      await client.get('/api/settings/server', true)
      return { success: true }
    } catch (error) {
      return {
        success: false,
        message: error instanceof Error ? error.message : 'Connection failed',
      }
    }
  },

  // Register device and get JWT token
  async registerDevice(
    serverUrl: string,
    deviceName: string,
    deviceType: 'mobile' | 'desktop'
  ): Promise<RegisterDeviceResponse> {
    const client = apiClient
    client.setBaseUrl(serverUrl)

    return client.post<RegisterDeviceResponse>(
      '/api/auth/register-device',
      {
        device_name: deviceName,
        device_type: deviceType,
      },
      true
    )
  },

  // Pair host with daemon
  async daemonHello(params: DaemonHelloRequest): Promise<void> {
    return apiClient.post('/api/auth/daemon-hello', params)
  },

  // Complete host pairing
  async completePair(params: PairHostRequest): Promise<PairResponse> {
    return apiClient.post<PairResponse>('/api/auth/pair', params)
  },
}
