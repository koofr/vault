import Dropdown from '@restart/ui/Dropdown';
import { memo, useState } from 'react';

import { LanguagePicker } from './LanguagePicker';
import { LanguagePickerMenu } from './LanguagePickerMenu';

export const LanguagePickerDropdown = memo(
  ({
    size = 'small',
    placement = 'top',
    dropdownToggleClassName,
  }: {
    size?: 'small' | 'large';
    placement?: 'top' | 'bottom';
    dropdownToggleClassName?: string;
  }) => {
    const [isVisible, setVisible] = useState(false);

    return (
      <Dropdown
        show={isVisible}
        onToggle={(value) => setVisible(value)}
        placement={placement}
      >
        <LanguagePicker
          size={size}
          dropdownToggleClassName={dropdownToggleClassName}
        />
        <LanguagePickerMenu />
      </Dropdown>
    );
  },
);
LanguagePickerDropdown.displayName = 'LanguagePickerDropdown';
