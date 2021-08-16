import { FormControl, InputLabel, Select, MenuItem, withStyles } from '@material-ui/core';
import { Attribute } from 'gw2-ui-bulk';
import React from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { getProfession, getControl, changeControl } from '../../state/gearOptimizerSlice';
import { Classes } from '../../utils/gw2-data';
import { firstUppercase } from '../../utils/usefulFunctions';

const styles = (theme) => ({
  root: {
    width: '100%',
  },
  gw2Item: {
    fontSize: '20px',
    color: '#AAAAAA',
  },
  formControl: {
    width: 100,
  },
});

const WeaponSelect = ({ classes }) => {
  const dispatch = useDispatch();
  const profession = useSelector(getProfession);
  const wea1mh = useSelector(getControl('wea1mh'));
  const wea1oh = useSelector(getControl('wea1oh'));

  const classData = Classes[profession.toLowerCase()];

  const canOffhand1 =
    classData.weapons.mainHand.find((w) => w.name === wea1mh).type !== 'twoHanded';

  console.log(canOffhand1);
  console.log(classData);

  return (
    <>
      <FormControl className={classes.formControl}>
        <InputLabel id="demo-simple-select-label">1. Wea MH</InputLabel>
        <Select
          labelId="demo-simple-select-label"
          id="demo-simple-select"
          value={wea1mh || ''}
          onChange={(e) => dispatch(changeControl({ key: 'wea1mh', value: e.target.value }))}
        >
          {classData.weapons.mainHand.map((wea) => (
            <MenuItem value={wea.name}>{wea.name}</MenuItem>
          ))}
        </Select>
      </FormControl>

      <FormControl className={classes.formControl}>
        <InputLabel id="demo-simple-select-label">1. Wea OH</InputLabel>
        <Select
          labelId="demo-simple-select-label"
          id="demo-simple-select"
          value={wea1oh || ''}
          onChange={(e) => dispatch(changeControl({ key: 'wea1oh', value: e.target.value }))}
        >
          {classData.weapons.offHand.map((wea) => (
            <MenuItem value={wea.name}>{wea.name}</MenuItem>
          ))}
        </Select>
      </FormControl>
    </>
  );
};

export default withStyles(styles)(WeaponSelect);
