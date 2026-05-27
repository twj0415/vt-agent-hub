import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { createI18n } from 'vue-i18n'
import { defineComponent, h } from 'vue'
import SchemaForm from './SchemaForm.vue'

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  messages: {
    'en-US': {},
  },
})

const AForm = defineComponent({
  setup(_, { slots, attrs }) {
    return () => h('form', attrs, slots.default?.())
  },
})

const AFormItem = defineComponent({
  props: {
    label: { type: String, default: '' },
  },
  setup(props, { slots, attrs }) {
    return () => h('label', attrs, [props.label, slots.default?.(), slots.extra?.()])
  },
})

const AInput = defineComponent({
  props: {
    value: { type: String, default: '' },
  },
  emits: ['update:value'],
  setup(props, { emit, attrs }) {
    return () => h('input', {
      ...attrs,
      value: props.value,
      onInput: (event: Event) => emit('update:value', (event.target as HTMLInputElement).value),
    })
  },
})

const ATextarea = defineComponent({
  props: {
    value: { type: String, default: '' },
  },
  emits: ['update:value'],
  setup(props, { emit, attrs }) {
    return () => h('textarea', {
      ...attrs,
      value: props.value,
      onInput: (event: Event) => emit('update:value', (event.target as HTMLTextAreaElement).value),
    })
  },
})

const ASelect = defineComponent({
  props: {
    value: { type: String, default: '' },
    options: { type: Array, default: () => [] },
  },
  emits: ['update:value'],
  setup(props, { emit, attrs }) {
    return () => h('select', {
      ...attrs,
      value: props.value,
      onChange: (event: Event) => emit('update:value', (event.target as HTMLSelectElement).value),
    }, (props.options as Array<{ label: string, value: string }>).map((option) =>
      h('option', { value: option.value }, option.label),
    ))
  },
})

describe('SchemaForm', () => {
  it('emits updateField when input changes', async () => {
    const wrapper = mount(SchemaForm, {
      props: {
        fields: [
          { key: 'profile', label: 'Profile', value: 'codex-default' },
        ],
      },
      global: {
        plugins: [i18n],
        components: {
          AForm,
          AFormItem,
          AInput,
          AInputPassword: AInput,
          ASelect,
          ATextarea,
        },
      },
    })

    await wrapper.get('input').setValue('codex-custom')

    expect(wrapper.emitted('updateField')).toEqual([
      [{ key: 'profile', value: 'codex-custom' }],
    ])
  })

  it('emits updateField when select changes', async () => {
    const wrapper = mount(SchemaForm, {
      props: {
        fields: [
          {
            key: 'model',
            label: 'Model',
            type: 'select',
            value: 'gpt-5.5',
            options: [
              { label: 'GPT-5.5', value: 'gpt-5.5' },
              { label: 'GPT-5.4 Mini', value: 'gpt-5.4-mini' },
            ],
          },
        ],
      },
      global: {
        plugins: [i18n],
        components: {
          AForm,
          AFormItem,
          AInput,
          AInputPassword: AInput,
          ASelect,
          ATextarea,
        },
      },
    })

    await wrapper.get('select').setValue('gpt-5.4-mini')

    expect(wrapper.emitted('updateField')).toEqual([
      [{ key: 'model', value: 'gpt-5.4-mini' }],
    ])
  })
})
