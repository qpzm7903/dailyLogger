<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-xl border border-gray-700 w-full max-w-md p-6">
      <!-- Tabs -->
      <div class="flex gap-2 mb-6">
        <button
          @click="mode = 'login'"
          :class="mode === 'login' ? 'bg-primary text-white' : 'bg-gray-700 text-gray-300'"
          class="flex-1 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {{ t('auth.login') }}
        </button>
        <button
          @click="mode = 'register'"
          :class="mode === 'register' ? 'bg-primary text-white' : 'bg-gray-700 text-gray-300'"
          class="flex-1 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {{ t('auth.register') }}
        </button>
      </div>

      <!-- Login Form -->
      <form v-if="mode === 'login'" @submit.prevent="handleLogin" class="space-y-4">
        <div>
          <label class="block text-sm text-gray-400 mb-1">{{ t('auth.username') }}</label>
          <input
            v-model="loginForm.username"
            type="text"
            class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary"
            :placeholder="t('auth.usernamePlaceholder')"
            required
          />
        </div>
        <div>
          <label class="block text-sm text-gray-400 mb-1">{{ t('auth.password') }}</label>
          <input
            v-model="loginForm.password"
            type="password"
            class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary"
            :placeholder="t('auth.passwordPlaceholder')"
            required
          />
        </div>
        <div v-if="error" class="text-red-400 text-sm">{{ error }}</div>
        <button
          type="submit"
          :disabled="loading"
          class="w-full bg-primary hover:bg-blue-600 disabled:opacity-50 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {{ loading ? t('auth.loggingIn') : t('auth.login') }}
        </button>
      </form>

      <!-- Register Form -->
      <form v-else @submit.prevent="handleRegister" class="space-y-4">
        <div>
          <label class="block text-sm text-gray-400 mb-1">{{ t('auth.username') }}</label>
          <input
            v-model="registerForm.username"
            type="text"
            class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary"
            :placeholder="t('auth.usernamePlaceholder')"
            required
          />
          <p class="text-xs text-gray-500 mt-1">{{ t('auth.usernameHint') }}</p>
        </div>
        <div>
          <label class="block text-sm text-gray-400 mb-1">{{ t('auth.email') }}</label>
          <input
            v-model="registerForm.email"
            type="email"
            class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary"
            :placeholder="t('auth.emailPlaceholder')"
          />
          <p class="text-xs text-gray-500 mt-1">{{ t('auth.emailHint') }}</p>
        </div>
        <div>
          <label class="block text-sm text-gray-400 mb-1">{{ t('auth.password') }}</label>
          <input
            v-model="registerForm.password"
            type="password"
            class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary"
            :placeholder="t('auth.passwordPlaceholder')"
            required
          />
          <p class="text-xs text-gray-500 mt-1">{{ t('auth.passwordHint') }}</p>
        </div>
        <div>
          <label class="block text-sm text-gray-400 mb-1">{{ t('auth.confirmPassword') }}</label>
          <input
            v-model="registerForm.confirmPassword"
            type="password"
            class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary"
            :placeholder="t('auth.confirmPasswordPlaceholder')"
            required
          />
        </div>
        <div v-if="error" class="text-red-400 text-sm">{{ error }}</div>
        <button
          type="submit"
          :disabled="loading"
          class="w-full bg-primary hover:bg-blue-600 disabled:opacity-50 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {{ loading ? t('auth.registering') : t('auth.register') }}
        </button>
      </form>

      <!-- Close button -->
      <button
        @click="$emit('close')"
        class="absolute top-4 right-4 text-gray-400 hover:text-white"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'loggedIn', user: { id: string; username: string }): void;
}>();

const mode = ref<'login' | 'register'>('login');
const loading = ref(false);
const error = ref('');

const loginForm = reactive({
  username: '',
  password: '',
});

const registerForm = reactive({
  username: '',
  email: '',
  password: '',
  confirmPassword: '',
});

async function handleLogin() {
  loading.value = true;
  error.value = '';

  try {
    const user = await invoke<{ id: string; username: string }>('login_user', {
      params: {
        username: loginForm.username,
        password: loginForm.password,
      },
    });
    emit('loggedIn', user);
    emit('close');
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function handleRegister() {
  loading.value = true;
  error.value = '';

  // Validate passwords match
  if (registerForm.password !== registerForm.confirmPassword) {
    error.value = t('auth.passwordMismatch');
    loading.value = false;
    return;
  }

  try {
    const user = await invoke<{ id: string; username: string }>('register_user', {
      params: {
        username: registerForm.username,
        email: registerForm.email || null,
        password: registerForm.password,
      },
    });
    emit('loggedIn', user);
    emit('close');
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}
</script>