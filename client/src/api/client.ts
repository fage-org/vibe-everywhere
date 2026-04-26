import { useAuthStore } from '@/stores/auth'
import type { ApiError } from './types'

export class ApiClientError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string
  ) {
    super(message)
    this.name = 'ApiClientError'
  }
}

class ApiClient {
  private baseUrl: string = ''

  setBaseUrl(url: string) {
    this.baseUrl = url.replace(/\/$/, '')
  }

  getBaseUrl(): string {
    return this.baseUrl
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    skipAuth: boolean = false
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    }

    if (!skipAuth) {
      const authStore = useAuthStore()
      if (authStore.token) {
        headers['Authorization'] = `Bearer ${authStore.token}`
      }
    }

    try {
      const response = await fetch(url, {
        method,
        headers,
        body: body ? JSON.stringify(body) : undefined,
      })

      // Handle 401 - token expired or invalid
      if (response.status === 401) {
        const authStore = useAuthStore()
        authStore.logout()
        throw new ApiClientError('Unauthorized', 401, 'UNAUTHORIZED')
      }

      // Parse response
      let data: unknown
      const contentType = response.headers.get('content-type')
      if (contentType?.includes('application/json')) {
        data = await response.json()
      } else {
        data = await response.text()
      }

      if (!response.ok) {
        const errorData = data as ApiError
        throw new ApiClientError(
          errorData.error || `HTTP ${response.status}`,
          response.status,
          errorData.code
        )
      }

      return data as T
    } catch (error) {
      if (error instanceof ApiClientError) {
        throw error
      }

      // Network or other errors
      throw new ApiClientError(
        error instanceof Error ? error.message : 'Network error',
        undefined,
        'NETWORK_ERROR'
      )
    }
  }

  get<T>(path: string, skipAuth?: boolean): Promise<T> {
    return this.request<T>('GET', path, undefined, skipAuth)
  }

  post<T>(path: string, body?: unknown, skipAuth?: boolean): Promise<T> {
    return this.request<T>('POST', path, body, skipAuth)
  }

  put<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PUT', path, body)
  }

  delete<T>(path: string): Promise<T> {
    return this.request<T>('DELETE', path)
  }
}

export const apiClient = new ApiClient()
