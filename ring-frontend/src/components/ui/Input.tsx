import './Input.css'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  input_type?: 'input' | 'textarea' | 'select'
}

export function Input({ input_type = 'input', className = '', ...props }: InputProps) {
  const cls = `input-field ${className}`
  if (input_type === 'textarea') {
    return <textarea className={`${cls} textarea-field`} {...(props as React.TextareaHTMLAttributes<HTMLTextAreaElement>)} />
  }
  if (input_type === 'select') {
    return <select className={cls} {...(props as React.SelectHTMLAttributes<HTMLSelectElement>)} />
  }
  return <input className={cls} {...props} />
}
