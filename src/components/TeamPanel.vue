<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-gray-300">{{ t('team.title') }}</h3>
      <div class="flex gap-2">
        <button
          @click="showJoinDialog = true"
          class="px-3 py-1 text-sm bg-gray-700 text-white rounded hover:bg-gray-600 transition"
        >
          {{ t('team.joinTeam') }}
        </button>
        <button
          @click="showCreateDialog = true"
          class="px-3 py-1 text-sm bg-primary text-darker rounded hover:bg-opacity-80 transition"
        >
          {{ t('team.createTeam') }}
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-4 text-gray-400">
      {{ t('common.loading') }}
    </div>

    <!-- Empty State -->
    <div v-else-if="teams.length === 0" class="text-center py-4 text-gray-400">
      {{ t('team.noTeams') }}
    </div>

    <!-- Team List -->
    <div v-else class="space-y-3">
      <div
        v-for="team in teams"
        :key="team.team.id"
        class="bg-darker rounded-lg p-4 border border-gray-700"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <div class="flex items-center gap-2">
              <h4 class="text-primary font-medium">{{ team.team.name }}</h4>
              <span
                class="text-xs px-2 py-0.5 rounded"
                :class="{
                  'bg-green-900 text-green-300': team.current_user_role === 'admin',
                  'bg-blue-900 text-blue-300': team.current_user_role === 'member',
                  'bg-gray-700 text-gray-300': team.current_user_role === 'viewer'
                }"
              >
                {{ getRoleLabel(team.current_user_role) }}
              </span>
            </div>
            <p v-if="team.team.description" class="text-sm text-gray-400 mt-1">
              {{ team.team.description }}
            </p>
            <p class="text-xs text-gray-500 mt-2">
              {{ t('team.members') }}: {{ team.members.length }} |
              {{ t('team.created') }}: {{ formatDate(team.team.created_at) }}
            </p>
          </div>
          <div class="flex gap-2">
            <button
              @click="showTeamDetails(team)"
              class="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded transition"
            >
              {{ t('team.viewDetails') }}
            </button>
            <button
              v-if="team.current_user_role === 'admin'"
              @click="showInviteCode(team)"
              class="px-2 py-1 text-xs bg-blue-700 hover:bg-blue-600 rounded transition"
            >
              {{ t('team.inviteCode') }}
            </button>
            <button
              v-if="team.team.owner_id !== currentUserId"
              @click="handleLeaveTeam(team.team.id)"
              :disabled="leavingTeam === team.team.id"
              class="px-2 py-1 text-xs bg-red-700 hover:bg-red-600 disabled:opacity-50 rounded transition"
            >
              {{ leavingTeam === team.team.id ? t('common.loading') : t('team.leave') }}
            </button>
            <button
              v-if="team.team.owner_id === currentUserId"
              @click="handleDeleteTeam(team.team.id)"
              :disabled="deletingTeam === team.team.id"
              class="px-2 py-1 text-xs bg-red-700 hover:bg-red-600 disabled:opacity-50 rounded transition"
            >
              {{ deletingTeam === team.team.id ? t('common.loading') : t('team.delete') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Team Dialog -->
    <div
      v-if="showCreateDialog"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
      @click.self="showCreateDialog = false"
    >
      <div class="bg-dark rounded-xl border border-gray-700 w-full max-w-md p-6">
        <h3 class="text-lg font-semibold mb-4">{{ t('team.createTeam') }}</h3>
        <form @submit.prevent="handleCreateTeam" class="space-y-4">
          <div>
            <label class="block text-sm text-gray-400 mb-1">{{ t('team.teamName') }}</label>
            <input
              v-model="createForm.name"
              type="text"
              class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary"
              :placeholder="t('team.teamNamePlaceholder')"
              required
            />
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">{{ t('team.teamDescription') }}</label>
            <textarea
              v-model="createForm.description"
              class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary resize-none"
              :placeholder="t('team.teamDescriptionPlaceholder')"
              rows="3"
            />
          </div>
          <div v-if="createError" class="text-red-400 text-sm">{{ createError }}</div>
          <div class="flex gap-3 justify-end">
            <button
              type="button"
              @click="showCreateDialog = false"
              class="px-4 py-2 text-sm bg-gray-700 rounded-lg hover:bg-gray-600 transition"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              :disabled="creating"
              class="px-4 py-2 text-sm bg-primary text-darker rounded-lg hover:bg-opacity-80 disabled:opacity-50 transition"
            >
              {{ creating ? t('common.loading') : t('team.createTeam') }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Join Team Dialog -->
    <div
      v-if="showJoinDialog"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
      @click.self="showJoinDialog = false"
    >
      <div class="bg-dark rounded-xl border border-gray-700 w-full max-w-md p-6">
        <h3 class="text-lg font-semibold mb-4">{{ t('team.joinTeam') }}</h3>
        <form @submit.prevent="handleJoinTeam" class="space-y-4">
          <div>
            <label class="block text-sm text-gray-400 mb-1">{{ t('team.inviteCodeLabel') }}</label>
            <input
              v-model="joinCode"
              type="text"
              class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-primary uppercase tracking-wider"
              :placeholder="t('team.inviteCodePlaceholder')"
              maxlength="8"
              required
            />
          </div>
          <div v-if="joinError" class="text-red-400 text-sm">{{ joinError }}</div>
          <div class="flex gap-3 justify-end">
            <button
              type="button"
              @click="showJoinDialog = false"
              class="px-4 py-2 text-sm bg-gray-700 rounded-lg hover:bg-gray-600 transition"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              :disabled="joining"
              class="px-4 py-2 text-sm bg-primary text-darker rounded-lg hover:bg-opacity-80 disabled:opacity-50 transition"
            >
              {{ joining ? t('common.loading') : t('team.joinTeam') }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Team Details Dialog -->
    <div
      v-if="showDetailsDialog && selectedTeam"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
      @click.self="showDetailsDialog = false"
    >
      <div class="bg-dark rounded-xl border border-gray-700 w-full max-w-lg p-6 max-h-[80vh] overflow-y-auto">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold">{{ selectedTeam.team.name }}</h3>
          <button @click="showDetailsDialog = false" class="text-gray-400 hover:text-white">✕</button>
        </div>

        <!-- Invite Code (Admin only) -->
        <div v-if="selectedTeam.current_user_role === 'admin'" class="mb-4 p-3 bg-darker rounded-lg">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs text-gray-400">{{ t('team.inviteCodeLabel') }}</p>
              <p class="text-lg font-mono text-primary tracking-wider">{{ selectedTeam.team.invite_code }}</p>
            </div>
            <button
              @click="handleRegenerateCode(selectedTeam.team.id)"
              :disabled="regeneratingCode"
              class="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded transition"
            >
              {{ regeneratingCode ? t('common.loading') : t('team.regenerateCode') }}
            </button>
          </div>
        </div>

        <!-- Members List -->
        <h4 class="text-sm font-medium text-gray-300 mb-2">{{ t('team.members') }}</h4>
        <div class="space-y-2">
          <div
            v-for="member in selectedTeam.members"
            :key="member.user_id"
            class="flex items-center justify-between p-2 bg-darker rounded"
          >
            <div>
              <span class="text-white">{{ member.username }}</span>
              <span
                class="text-xs px-2 py-0.5 rounded ml-2"
                :class="{
                  'bg-green-900 text-green-300': member.role === 'admin',
                  'bg-blue-900 text-blue-300': member.role === 'member',
                  'bg-gray-700 text-gray-300': member.role === 'viewer'
                }"
              >
                {{ getRoleLabel(member.role) }}
              </span>
            </div>
            <div v-if="selectedTeam.current_user_role === 'admin' && member.user_id !== currentUserId" class="flex gap-1">
              <select
                @change="handleUpdateRole(selectedTeam.team.id, member.user_id, $event.target.value)"
                :value="member.role"
                class="text-xs bg-gray-700 border border-gray-600 rounded px-1 py-0.5 text-white"
              >
                <option value="admin">{{ t('team.roleAdmin') }}</option>
                <option value="member">{{ t('team.roleMember') }}</option>
                <option value="viewer">{{ t('team.roleViewer') }}</option>
              </select>
              <button
                @click="handleRemoveMember(selectedTeam.team.id, member.user_id)"
                class="text-xs text-red-400 hover:text-red-300 px-1"
              >
                {{ t('team.remove') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Close Button -->
        <div class="mt-4 flex justify-end">
          <button
            @click="showDetailsDialog = false"
            class="px-4 py-2 text-sm bg-gray-700 rounded-lg hover:bg-gray-600 transition"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showSuccess, showError } from '../stores/toast.js'

const { t } = useI18n()

const teams = ref([])
const loading = ref(true)
const currentUserId = ref(null)

// Dialog states
const showCreateDialog = ref(false)
const showJoinDialog = ref(false)
const showDetailsDialog = ref(false)

// Form data
const createForm = ref({ name: '', description: '' })
const joinCode = ref('')
const selectedTeam = ref(null)

// Loading states
const creating = ref(false)
const joining = ref(false)
const leavingTeam = ref(null)
const deletingTeam = ref(null)
const regeneratingCode = ref(false)

// Error states
const createError = ref('')
const joinError = ref('')

async function loadTeams() {
  loading.value = true
  try {
    // Get current session to get user ID
    const session = await invoke('get_current_session')
    if (session) {
      currentUserId.value = session.user_id
      const result = await invoke('get_user_teams', { userId: session.user_id })
      teams.value = Array.isArray(result) ? result : []
    } else {
      teams.value = []
    }
  } catch (error) {
    console.error('Failed to load teams:', error)
    teams.value = []
  } finally {
    loading.value = false
  }
}

async function handleCreateTeam() {
  if (!createForm.value.name.trim()) {
    createError.value = t('team.nameRequired')
    return
  }

  creating.value = true
  createError.value = ''
  try {
    await invoke('create_team', {
      params: {
        name: createForm.value.name.trim(),
        description: createForm.value.description.trim() || null,
        visibility: null
      },
      currentUserId: currentUserId.value
    })
    showSuccess(t('team.teamCreated'))
    showCreateDialog.value = false
    createForm.value = { name: '', description: '' }
    await loadTeams()
  } catch (error) {
    createError.value = error.toString()
  } finally {
    creating.value = false
  }
}

async function handleJoinTeam() {
  if (!joinCode.value.trim()) {
    joinError.value = t('team.codeRequired')
    return
  }

  joining.value = true
  joinError.value = ''
  try {
    await invoke('join_team', {
      inviteCode: joinCode.value.trim().toUpperCase(),
      currentUserId: currentUserId.value
    })
    showSuccess(t('team.joinedSuccess'))
    showJoinDialog.value = false
    joinCode.value = ''
    await loadTeams()
  } catch (error) {
    joinError.value = error.toString()
  } finally {
    joining.value = false
  }
}

async function handleLeaveTeam(teamId) {
  if (!confirm(t('team.confirmLeave'))) return

  leavingTeam.value = teamId
  try {
    await invoke('leave_team', {
      teamId,
      currentUserId: currentUserId.value
    })
    showSuccess(t('team.leftSuccess'))
    await loadTeams()
  } catch (error) {
    showError(error.toString())
  } finally {
    leavingTeam.value = null
  }
}

async function handleDeleteTeam(teamId) {
  if (!confirm(t('team.confirmDelete'))) return

  deletingTeam.value = teamId
  try {
    await invoke('delete_team', {
      teamId,
      currentUserId: currentUserId.value
    })
    showSuccess(t('team.deletedSuccess'))
    await loadTeams()
  } catch (error) {
    showError(error.toString())
  } finally {
    deletingTeam.value = null
  }
}

async function handleRegenerateCode(teamId) {
  regeneratingCode.value = true
  try {
    const newCode = await invoke('regenerate_invite_code', {
      teamId,
      currentUserId: currentUserId.value
    })
    selectedTeam.value.team.invite_code = newCode
    showSuccess(t('team.codeRegenerated'))
  } catch (error) {
    showError(error.toString())
  } finally {
    regeneratingCode.value = false
  }
}

async function handleUpdateRole(teamId, userId, newRole) {
  try {
    await invoke('update_member_role', {
      teamId,
      userId,
      newRole,
      currentUserId: currentUserId.value
    })
    showSuccess(t('team.roleUpdated'))
    await loadTeams()
    // Refresh selected team
    const team = teams.value.find(t => t.team.id === teamId)
    if (team) {
      selectedTeam.value = team
    }
  } catch (error) {
    showError(error.toString())
  }
}

async function handleRemoveMember(teamId, userId) {
  if (!confirm(t('team.confirmRemove'))) return

  try {
    await invoke('remove_member', {
      teamId,
      userId,
      currentUserId: currentUserId.value
    })
    showSuccess(t('team.memberRemoved'))
    await loadTeams()
    // Refresh selected team
    const team = teams.value.find(t => t.team.id === teamId)
    if (team) {
      selectedTeam.value = team
    }
  } catch (error) {
    showError(error.toString())
  }
}

function showTeamDetails(team) {
  selectedTeam.value = team
  showDetailsDialog.value = true
}

function showInviteCode(team) {
  selectedTeam.value = team
  showDetailsDialog.value = true
}

function getRoleLabel(role) {
  switch (role) {
    case 'admin': return t('team.roleAdmin')
    case 'member': return t('team.roleMember')
    case 'viewer': return t('team.roleViewer')
    default: return role
  }
}

function formatDate(dateStr) {
  try {
    const date = new Date(dateStr)
    return date.toLocaleDateString()
  } catch {
    return dateStr
  }
}

onMounted(() => {
  loadTeams()
})
</script>