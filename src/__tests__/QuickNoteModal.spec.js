/**
 * QuickNoteModal.vue 测试
 *
 * 覆盖行为：
 *  1. Enter 键触发 save 事件并携带已输入内容
 *  2. 内容为空（或纯空白）时不触发 save
 *  3. 点击保存按钮触发 save 事件
 *  4. 内容为空时保存按钮处于禁用状态
 *  5. 提示文字包含 Enter 与 Shift+Enter 说明
 */
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import QuickNoteModal from '../components/QuickNoteModal.vue'

describe('QuickNoteModal.vue - 笔记保存', () => {
  it('输入内容后按 Enter 触发 save 事件', async () => {
    const wrapper = mount(QuickNoteModal)

    await wrapper.find('textarea').setValue('今天完成了需求评审')
    await wrapper.find('textarea').trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('save')).toBeTruthy()
    expect(wrapper.emitted('save')[0]).toEqual(['今天完成了需求评审'])
  })

  it('内容为纯空白时 Enter 不触发 save', async () => {
    const wrapper = mount(QuickNoteModal)

    await wrapper.find('textarea').setValue('   ')
    await wrapper.find('textarea').trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('save')).toBeFalsy()
  })

  it('内容为空时 Enter 不触发 save', async () => {
    const wrapper = mount(QuickNoteModal)

    await wrapper.find('textarea').trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('save')).toBeFalsy()
  })

  it('点击保存按钮触发 save 事件', async () => {
    const wrapper = mount(QuickNoteModal)

    await wrapper.find('textarea').setValue('记录一个想法')
    await wrapper.find('button.bg-primary').trigger('click')

    expect(wrapper.emitted('save')).toBeTruthy()
    expect(wrapper.emitted('save')[0]).toEqual(['记录一个想法'])
  })

  it('内容为空时保存按钮禁用', () => {
    const wrapper = mount(QuickNoteModal)

    expect(wrapper.find('button.bg-primary').attributes('disabled')).toBeDefined()
  })

  it('内容不为空时保存按钮可用', async () => {
    const wrapper = mount(QuickNoteModal)

    await wrapper.find('textarea').setValue('有内容了')

    expect(wrapper.find('button.bg-primary').attributes('disabled')).toBeUndefined()
  })
})

describe('QuickNoteModal.vue - 快捷键提示', () => {
  it('提示文字包含"Enter 保存"说明', () => {
    const wrapper = mount(QuickNoteModal)

    expect(wrapper.text()).toContain('Enter 保存')
  })

  it('提示文字包含"Shift+Enter 换行"说明', () => {
    const wrapper = mount(QuickNoteModal)

    expect(wrapper.text()).toContain('Shift+Enter 换行')
  })
})
