export type FormActionResponse =
  | null
  | {
      success: true
      messages?: string[]
    }
  | {
      success: false
      messages: string[]
    }
